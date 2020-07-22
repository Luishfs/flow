use estuary::{
    catalog::{self, sql_params},
    derive, doc,
};
use estuary_protocol::flow;
use futures::{select, FutureExt};
use log::{error, info};
use pretty_env_logger;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tokio;
use tokio::signal::unix::{signal, SignalKind};
use tower::Service;
use url::Url;

#[derive(StructOpt, Debug)]
struct DeriveCommand {
    #[structopt(parse(from_os_str), help = "Path to the catalog database.")]
    catalog: PathBuf,
    #[structopt(help = "Name of the collection to derive.")]
    derivation: String,
    #[structopt(
        parse(from_os_str),
        help = "Unix domain socket to listen on for gRPC connections."
    )]
    grpc_socket_path: PathBuf,
}

#[derive(StructOpt, Debug)]
struct ExtractCommand {
    #[structopt(
        parse(from_os_str),
        help = "Unix domain socket to listen on for gRPC connections."
    )]
    grpc_socket_path: PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(
    author = "Estuary Technologies, Inc. \u{00A9}2020",
    about = "Worker side-car process of Estuary Flow, for deriving and extracting documents"
)]
enum Command {
    Derive(DeriveCommand),
    Extract(ExtractCommand),
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cmd = Command::from_args();

    println!("{:?}", cmd);

    let result = match cmd {
        Command::Extract(cmd) => cmd.run().await,
        Command::Derive(cmd) => cmd.run().await,
    };
    if let Err(err) = result {
        error!("exiting with error: {}", err);
    };
}

impl ExtractCommand {
    async fn run(&self) -> Result<(), Error> {
        let mut extract_api =
            flow::extract_server::ExtractServer::new(derive::extract::ExtractAPI {});
        let service =
            tower::service_fn(move |req: hyper::Request<hyper::Body>| extract_api.call(req));

        // Bind local listener and begin serving.
        let server = estuary::serve::unix_domain_socket(
            service,
            &self.grpc_socket_path,
            register_signal_handlers()?,
        );
        let server_handle = tokio::spawn(server);

        // Signal to host process that we're ready to accept connections.
        println!("READY");
        server_handle.await?;

        Ok(())
    }
}

impl DeriveCommand {
    async fn run(&self) -> Result<(), Error> {
        // Open catalog DB.
        let db = catalog::open(&self.catalog)?;

        let derivation_id = db
            .prepare(
                "SELECT collection_id
                FROM collections NATURAL JOIN derivations
                WHERE name = ?",
            )?
            .query_row(sql_params![self.derivation], |r| r.get(0))
            .map_err(|err| catalog::Error::At {
                loc: format!("querying for derived collection {:?}", self.derivation),
                detail: Box::new(err.into()),
            })?;

        // "Open" recovered state store, instrumented with a Recorder.
        // TODO rocksdb, sqlite, Go CGO bindings to client / Recorder, blah blah.
        let store = Box::new(derive::state::MemoryStore::new());
        let _store = Arc::new(Mutex::new(store));

        // Compile the bundle of catalog schemas. Then, deliberately "leak" the
        // immutable Schema bundle for the remainder of program in order to achieve
        // a 'static lifetime, which is required for use in spawned tokio Tasks (and
        // therefore in TxnCtx).
        let schemas = catalog::Schema::compile_all(&db)?;
        let schemas = Box::leak(Box::new(schemas));

        let mut schema_index = doc::SchemaIndex::<'static>::new();
        for schema in schemas.iter() {
            schema_index.add(schema)?;
        }
        schema_index.verify_references()?;

        info!("loaded {} JSON-Schemas from catalog", schemas.len());

        // Start NodeJS transform worker.
        let loopback = Url::from_file_path(&self.grpc_socket_path).unwrap();
        let node = derive::nodejs::Service::new(&db, loopback)?;

        let txn_ctx = derive::transform::Context::new(&db, derivation_id, node, schema_index)?;
        let txn_ctx = Arc::new(Box::new(txn_ctx));

        // Build service.
        let mut extract_svc =
            flow::extract_server::ExtractServer::new(derive::extract::ExtractAPI {});
        //let mut derive_svc = flow::derive_server::DeriveServer::new(derive::DeriveService {});

        let service = tower::service_fn(move |req: hyper::Request<hyper::Body>| {
            let path = &req.uri().path()[1..];

            if path.starts_with(grpc_service_name(&extract_svc)) {
                extract_svc.call(req)
            } else {
                extract_svc.call(req)
            }
        });

        // Bind local listener and begin serving.
        let server = estuary::serve::unix_domain_socket(
            service,
            &self.grpc_socket_path,
            register_signal_handlers()?,
        );
        let server_handle = tokio::spawn(server);

        // Invoke derivation bootstraps.
        txn_ctx.node.bootstrap(derivation_id).await?;

        // Signal to host process that we're ready to accept connections.
        println!("READY");
        server_handle.await?;

        Ok(())
    }
}

fn grpc_service_name<S: tonic::transport::NamedService>(_: &S) -> &str {
    return S::NAME;
}

fn register_signal_handlers() -> Result<impl std::future::Future<Output = ()>, Error> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    Ok(async move {
        select!(
            _ = sigterm.recv().fuse() => info!("caught SIGTERM; stopping"),
            _ = sigint.recv().fuse() => info!("caught SIGINT; stopping"),
        );
    })
}
