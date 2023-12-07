pub mod read;

use crate::Timestamp;
use anyhow::Context;
use journal_client::{fragments, list};
use proto_gazette::broker;
use time::OffsetDateTime;

use crate::dataplane::journal_client_for;
use crate::output::{to_table_row, CliOutput, JsonCell};

use self::read::ReadArgs;

/// Selector of collection journals, which is used for reads, journal and fragment listings, etc.
#[derive(clap::Args, Default, Debug, Clone)]
pub struct CollectionJournalSelector {
    /// The full name of the Flow collection
    #[clap(long)]
    pub collection: String,
    /// Selects a subset of collection partitions using the given selector.
    /// The selector is provided as JSON matching the same shape that's used
    /// in Flow catalog specs. For example:
    /// '{"include": {"myField1":["value1", "value2"]}}'
    #[clap(
        long,
        value_parser(parse_partition_selector),
        conflicts_with_all(&["include-partition", "exclude-partition"])
    )]
    pub partitions: Option<models::PartitionSelector>,

    /// Deprecated, use --partitions instead
    #[clap(long = "include-partition", value_parser(parse_deprecated_selector))]
    pub include_partitions: Vec<String>,
    /// Deprecated, use --partitions instead
    #[clap(long = "exclude-partition", value_parser(parse_deprecated_selector))]
    pub exclude_partitions: Vec<String>,
}

fn parse_deprecated_selector(_: &str) -> Result<String, anyhow::Error> {
    anyhow::bail!("this argument has been deprecated, and replaced by --partitions")
}

fn parse_partition_selector(arg: &str) -> Result<models::PartitionSelector, anyhow::Error> {
    serde_json::from_str(arg).context("parsing `--partitions` argument value")
}

impl CollectionJournalSelector {
    pub fn build_label_selector(&self) -> broker::LabelSelector {
        assemble::journal_selector(
            &models::Collection::new(&self.collection),
            self.partitions.as_ref(),
        )
    }
}

#[derive(clap::Args, Debug)]
pub struct Collections {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Read data from a Flow collection and output to stdout.
    Read(ReadArgs),
    /// List the individual journals of a flow collection
    ListJournals(CollectionJournalSelector),
    /// List the journal fragments of a flow collection
    ListFragments(ListFragmentsArgs),
}

impl Collections {
    pub async fn run(&self, ctx: &mut crate::CliContext) -> Result<(), anyhow::Error> {
        match &self.cmd {
            Command::Read(args) => do_read(ctx, args).await,
            Command::ListJournals(selector) => do_list_journals(ctx, selector).await,
            Command::ListFragments(args) => do_list_fragments(ctx, args).await,
        }
    }
}

async fn do_read(ctx: &mut crate::CliContext, args: &ReadArgs) -> Result<(), anyhow::Error> {
    tracing::debug!(?args, "executing read");
    read::read_collection(ctx, args).await?;
    Ok(())
}

impl CliOutput for broker::JournalSpec {
    type TableAlt = ();
    type CellValue = JsonCell;

    fn table_headers(_alt: Self::TableAlt) -> Vec<&'static str> {
        vec![
            "Name",
            "Max Append Rate",
            "Fragment Length",
            "Fragment Flush Interval",
            "Fragment Primary Store",
        ]
    }

    fn into_table_row(self, _alt: Self::TableAlt) -> Vec<Self::CellValue> {
        to_table_row(
            self,
            &[
                "/name",
                "/maxAppendRate",
                "/fragment/length",
                "/fragment/flushInterval",
                "/fragment/stores/0",
            ],
        )
    }
}

#[derive(clap::Args, Debug)]
pub struct ListFragmentsArgs {
    #[clap(flatten)]
    pub selector: CollectionJournalSelector,

    /// If provided, then the frament listing will include a pre-signed URL for each fragment, which is valid for the given duration.
    /// This can be used to fetch fragment data directly from cloud storage.
    #[clap(long)]
    pub signature_ttl: Option<humantime::Duration>,

    /// Only include fragments which were written within the provided duration from the present.
    /// For example, `--since 10m` will only output fragments that have been written within the last 10 minutes.
    #[clap(long)]
    pub since: Option<humantime::Duration>,
}

impl CliOutput for broker::fragments_response::Fragment {
    type TableAlt = bool;

    type CellValue = String;

    fn table_headers(signed_urls: Self::TableAlt) -> Vec<&'static str> {
        let mut headers = vec!["Journal", "Begin Offset", "Size", "Store", "Mod Time"];
        if signed_urls {
            headers.push("Signed URL");
        }
        headers
    }

    fn into_table_row(mut self, signed_urls: Self::TableAlt) -> Vec<Self::CellValue> {
        let spec = self.spec.take().expect("missing spec of FragmentsResponse");
        let store = if spec.backing_store.is_empty() {
            String::new()
        } else {
            format!(
                "{}{}{}",
                spec.backing_store, spec.journal, spec.path_postfix
            )
        };

        let size = ::size::Size::from_bytes(spec.end - spec.begin);

        let mod_timestamp = if spec.mod_time > 0 {
            Timestamp::from_unix_timestamp(spec.mod_time)
                .map(|ts| ts.to_string())
                .unwrap_or_else(|_| {
                    tracing::error!(
                        mod_time = spec.mod_time,
                        "fragment has invalid mod_time value"
                    );
                    String::from("invalid timestamp")
                })
        } else {
            String::new()
        };
        let mut columns = vec![
            spec.journal,
            spec.begin.to_string(),
            size.to_string(),
            store,
            mod_timestamp,
        ];
        if signed_urls {
            columns.push(self.signed_url);
        }
        columns
    }
}

async fn do_list_fragments(
    ctx: &mut crate::CliContext,
    args: &ListFragmentsArgs,
) -> Result<(), anyhow::Error> {
    let mut client = journal_client_for(
        ctx.controlplane_client().await?,
        vec![args.selector.collection.clone()],
    )
    .await?;

    let journals = list::list_journals(&mut client, &args.selector.build_label_selector()).await?;

    let start_time = if let Some(since) = args.since {
        let timepoint = OffsetDateTime::now_utc() - *since;
        tracing::debug!(%since, begin_mod_time = %timepoint, "resolved --since to begin_mod_time");
        timepoint.unix_timestamp()
    } else {
        0
    };

    let signature_ttl = args
        .signature_ttl
        .map(|ttl| std::time::Duration::from(*ttl).into());
    let mut fragments = Vec::with_capacity(32);
    for journal in journals {
        let req = broker::FragmentsRequest {
            journal: journal.name.clone(),
            begin_mod_time: start_time,
            page_limit: 500,
            signature_ttl: signature_ttl.clone(),
            ..Default::default()
        };

        let mut fragment_iter = fragments::FragmentIter::new(client.clone(), req);

        while let Some(fragment) = fragment_iter.next().await {
            fragments.push(fragment?);
        }
    }

    ctx.write_all(fragments, args.signature_ttl.is_some())
}

async fn do_list_journals(
    ctx: &mut crate::CliContext,
    args: &CollectionJournalSelector,
) -> Result<(), anyhow::Error> {
    let mut client = journal_client_for(
        ctx.controlplane_client().await?,
        vec![args.collection.clone()],
    )
    .await?;

    let journals = list::list_journals(&mut client, &args.build_label_selector()).await?;

    ctx.write_all(journals, ())
}
