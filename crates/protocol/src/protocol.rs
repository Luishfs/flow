/// Label defines a key & value pair which can be attached to entities like
/// JournalSpecs and BrokerSpecs. Labels may be used to provide identifying
/// attributes which do not directly imply semantics to the core system, but
/// are meaningful to users or for higher-level Gazette tools.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Label {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
/// LabelSet is a collection of labels and their values.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelSet {
    /// Labels of the set. Instances must be unique and sorted over (Name, Value).
    #[prost(message, repeated, tag = "1")]
    pub labels: ::prost::alloc::vec::Vec<Label>,
}
/// LabelSelector defines a filter over LabelSets.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelSelector {
    /// Include is Labels which must be matched for a LabelSet to be selected. If
    /// empty, all Labels are included. An include Label with empty ("") value is
    /// matched by a Label of the same name having any value.
    #[prost(message, optional, tag = "1")]
    pub include: ::core::option::Option<LabelSet>,
    /// Exclude is Labels which cannot be matched for a LabelSet to be selected. If
    /// empty, no Labels are excluded. An exclude Label with empty ("") value
    /// excludes a Label of the same name having any value.
    #[prost(message, optional, tag = "2")]
    pub exclude: ::core::option::Option<LabelSet>,
}
/// JournalSpec describes a Journal and its configuration.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JournalSpec {
    /// Name of the Journal.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Desired replication of this Journal. This defines the Journal's tolerance
    /// to broker failures before data loss can occur (eg, a replication factor
    /// of three means two failures are tolerated).
    #[prost(int32, tag = "2")]
    pub replication: i32,
    /// User-defined Labels of this JournalSpec. Two label names are reserved
    /// and may not be used within a JournalSpec's Labels: "name" and "prefix".
    #[prost(message, optional, tag = "3")]
    pub labels: ::core::option::Option<LabelSet>,
    #[prost(message, optional, tag = "4")]
    pub fragment: ::core::option::Option<journal_spec::Fragment>,
    /// Flags of the Journal, as a combination of Flag enum values. The Flag enum
    /// is not used directly, as protobuf enums do not allow for or'ed bitfields.
    #[prost(uint32, tag = "6")]
    pub flags: u32,
    /// Maximum rate, in bytes-per-second, at which appends of this journal will
    /// be processed. If zero (the default), no rate limiting is applied. A global
    /// rate limit still may be in effect, in which case the effective rate is the
    /// smaller of the journal vs global rate.
    #[prost(int64, tag = "7")]
    pub max_append_rate: i64,
}
/// Nested message and enum types in `JournalSpec`.
pub mod journal_spec {
    /// Fragment is JournalSpec configuration which pertains to the creation,
    /// persistence, and indexing of the Journal's Fragments.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Fragment {
        /// Target content length of each Fragment. In normal operation after
        /// Fragments reach at least this length, they will be closed and new ones
        /// begun. Note lengths may be smaller at times (eg, due to changes in
        /// Journal routing topology). Content length differs from Fragment file
        /// size, in that the former reflects uncompressed bytes.
        #[prost(int64, tag = "1")]
        pub length: i64,
        /// Codec used to compress Journal Fragments.
        #[prost(enumeration = "super::CompressionCodec", tag = "2")]
        pub compression_codec: i32,
        /// Storage backend base path for this Journal's Fragments. Must be in URL
        /// form, with the choice of backend defined by the scheme. The full path of
        /// a Journal's Fragment is derived by joining the store path with the
        /// Fragment's ContentPath. Eg, given a fragment_store of
        ///   "s3://My-AWS-bucket/a/prefix" and a JournalSpec of name "my/journal",
        /// a complete Fragment path might be:
        ///   "s3://My-AWS-bucket/a/prefix/my/journal/000123-000456-789abcdef.gzip
        ///
        /// Multiple stores may be specified, in which case the Journal's Fragments
        /// are the union of all Fragments present across all stores, and new
        /// Fragments always persist to the first specified store. This can be
        /// helpful in performing incremental migrations, where new Journal content
        /// is written to the new store, while content in the old store remains
        /// available (and, depending on fragment_retention or recovery log pruning,
        /// may eventually be removed).
        ///
        /// If no stores are specified, the Journal is still use-able but will
        /// not persist Fragments to any a backing fragment store. This allows for
        /// real-time streaming use cases where reads of historical data are not
        /// needed.
        #[prost(string, repeated, tag = "3")]
        pub stores: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// Interval of time between refreshes of remote Fragment listings from
        /// configured fragment_stores.
        #[prost(message, optional, tag = "4")]
        pub refresh_interval: ::core::option::Option<::prost_types::Duration>,
        /// Retention duration for historical Fragments of this Journal within the
        /// Fragment stores. If less than or equal to zero, Fragments are retained
        /// indefinitely.
        #[prost(message, optional, tag = "5")]
        pub retention: ::core::option::Option<::prost_types::Duration>,
        /// Flush interval defines a uniform UTC time segment which, when passed,
        /// will prompt brokers to close and persist a fragment presently being
        /// written.
        ///
        /// Flush interval may be helpful in integrating the journal with a regularly
        /// scheduled batch work-flow which processes new files from the fragment
        /// store and has no particular awareness of Gazette. For example, setting
        /// flush_interval to 3600s will cause brokers to persist their present
        /// fragment on the hour, every hour, even if it has not yet reached its
        /// target length. A batch work-flow running at 5 minutes past the hour is
        /// then reasonably assured of seeing all events from the past hour.
        ///
        /// See also "gazctl journals fragments --help" for more discussion.
        #[prost(message, optional, tag = "6")]
        pub flush_interval: ::core::option::Option<::prost_types::Duration>,
        /// Path postfix template is a Go template which evaluates to a partial
        /// path under which fragments are persisted to the store. A complete
        /// fragment path is constructed by appending path components from the
        /// fragment store, then the journal name, and then the postfix template.
        /// Path post-fixes can help in maintaining Hive compatible partitioning
        /// over fragment creation time. The fields ".Spool" and ".JournalSpec"
        /// are available for introspection in the template. For example,
        /// to partition on the UTC date and hour of creation, use:
        ///
        ///    date={{ .Spool.FirstAppendTime.Format "2006-01-02" }}/hour={{
        ///    .Spool.FirstAppendTime.Format "15" }}
        ///
        /// Which will produce a path postfix like "date=2019-11-19/hour=22".
        #[prost(string, tag = "7")]
        pub path_postfix_template: ::prost::alloc::string::String,
    }
    /// Flags define Journal IO control behaviors. Where possible, flags are named
    /// after an equivalent POSIX flag.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Flag {
        /// NOT_SPECIFIED is considered as equivalent to O_RDWR by the broker. When
        /// JournalSpecs are union'ed (eg, by the `journalspace` pkg), NOT_SPECIFIED
        /// is considered as unset relative to any other non-zero Flag value.
        NotSpecified = 0,
        // Only one of O_RDONLY, O_WRONLY, or O_RDWR may be set.
        /// The Journal is available for reads (only).
        ORdonly = 1,
        /// The Journal is available for writes (only).
        OWronly = 2,
        /// The Journal may be used for reads or writes.
        ORdwr = 4,
    }
}
/// ProcessSpec describes a uniquely identified process and its addressable
/// endpoint.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessSpec {
    #[prost(message, optional, tag = "1")]
    pub id: ::core::option::Option<process_spec::Id>,
    /// Advertised URL of the process.
    #[prost(string, tag = "2")]
    pub endpoint: ::prost::alloc::string::String,
}
/// Nested message and enum types in `ProcessSpec`.
pub mod process_spec {
    /// ID composes a zone and a suffix to uniquely identify a ProcessSpec.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Id {
        /// "Zone" in which the process is running. Zones may be AWS, Azure, or
        /// Google Cloud Platform zone identifiers, or rack locations within a colo,
        /// or given some other custom meaning. Gazette will replicate across
        /// multiple zones, and seeks to minimize traffic which must cross zones (for
        /// example, by proxying reads to a broker in the current zone).
        #[prost(string, tag = "1")]
        pub zone: ::prost::alloc::string::String,
        /// Unique suffix of the process within |zone|. It is permissible for a
        /// suffix value to repeat across zones, but never within zones. In practice,
        /// it's recommended to use a FQDN, Kubernetes Pod name, or comparable unique
        /// and self-describing value as the ID suffix.
        #[prost(string, tag = "2")]
        pub suffix: ::prost::alloc::string::String,
    }
}
/// BrokerSpec describes a Gazette broker and its configuration.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BrokerSpec {
    /// ProcessSpec of the broker.
    #[prost(message, optional, tag = "1")]
    pub process_spec: ::core::option::Option<ProcessSpec>,
    /// Maximum number of assigned Journal replicas.
    #[prost(uint32, tag = "2")]
    pub journal_limit: u32,
}
/// Fragment is a content-addressed description of a contiguous Journal span,
/// defined by the [begin, end) offset range covered by the Fragment and the
/// SHA1 sum of the corresponding Journal content.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fragment {
    /// Journal of the Fragment.
    #[prost(string, tag = "1")]
    pub journal: ::prost::alloc::string::String,
    /// Begin (inclusive) and end (exclusive) offset of the Fragment within the
    /// Journal.
    #[prost(int64, tag = "2")]
    pub begin: i64,
    #[prost(int64, tag = "3")]
    pub end: i64,
    /// SHA1 sum of the Fragment's content.
    #[prost(message, optional, tag = "4")]
    pub sum: ::core::option::Option<Sha1Sum>,
    /// Codec with which the Fragment's content is compressed.
    #[prost(enumeration = "CompressionCodec", tag = "5")]
    pub compression_codec: i32,
    /// Fragment store which backs the Fragment. Empty if the Fragment has yet to
    /// be persisted and is still local to a Broker.
    #[prost(string, tag = "6")]
    pub backing_store: ::prost::alloc::string::String,
    /// Modification timestamp of the Fragment within the backing store,
    /// represented as seconds since the epoch.
    #[prost(int64, tag = "7")]
    pub mod_time: i64,
    /// Path postfix under which the fragment is persisted to the store.
    /// The complete Fragment store path is built from any path components of the
    /// backing store, followed by the journal name, followed by the path postfix.
    #[prost(string, tag = "8")]
    pub path_postfix: ::prost::alloc::string::String,
}
/// SHA1Sum is a 160-bit SHA1 digest.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sha1Sum {
    #[prost(fixed64, tag = "1")]
    pub part1: u64,
    #[prost(fixed64, tag = "2")]
    pub part2: u64,
    #[prost(fixed32, tag = "3")]
    pub part3: u32,
}
/// ReadRequest is the unary request message of the broker Read RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadRequest {
    /// Header is attached by a proxying broker peer.
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    /// Journal to be read.
    #[prost(string, tag = "2")]
    pub journal: ::prost::alloc::string::String,
    /// Desired offset to begin reading from. Value -1 has special handling, where
    /// the read is performed from the current write head. All other positive
    /// values specify a desired exact byte offset to read from. If the offset is
    /// not available (eg, because it represents a portion of Journal which has
    /// been permanently deleted), the broker will return the next available
    /// offset. Callers should therefore always inspect the ReadResponse offset.
    #[prost(int64, tag = "3")]
    pub offset: i64,
    /// Whether the operation should block until content becomes available.
    /// OFFSET_NOT_YET_AVAILABLE is returned if a non-blocking read has no ready
    /// content.
    #[prost(bool, tag = "4")]
    pub block: bool,
    /// If do_not_proxy is true, the broker will not proxy the read to another
    /// broker, or open and proxy a remote Fragment on the client's behalf.
    #[prost(bool, tag = "5")]
    pub do_not_proxy: bool,
    /// If metadata_only is true, the broker will respond with Journal and
    /// Fragment metadata but not content.
    #[prost(bool, tag = "6")]
    pub metadata_only: bool,
    /// Offset to read through. If zero, then the read end offset is unconstrained.
    #[prost(int64, tag = "7")]
    pub end_offset: i64,
}
/// ReadResponse is the streamed response message of the broker Read RPC.
/// Responses messages are of two types:
///
/// * "Metadata" messages, which conveys the journal Fragment addressed by the
///    request which is ready to be read.
/// * "Chunk" messages, which carry associated journal Fragment content bytes.
///
/// A metadata message specifying a Fragment always precedes all "chunks" of the
/// Fragment's content. Response streams may be very long lived, having many
/// metadata and accompanying chunk messages. The reader may also block for long
/// periods of time awaiting the next metadata message (eg, if the next offset
/// hasn't yet committed). However once a metadata message is read, the reader
/// is assured that its associated chunk messages are immediately forthcoming.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadResponse {
    /// Status of the Read RPC.
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    /// Header of the response. Accompanies the first ReadResponse of the response
    /// stream.
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<Header>,
    /// The effective offset of the read. See ReadRequest offset.
    #[prost(int64, tag = "3")]
    pub offset: i64,
    /// The offset to next be written, by the next append transaction served by
    /// broker. In other words, the last offset through which content is
    /// available to be read from the Journal. This is a metadata field and will
    /// not be returned with a content response.
    #[prost(int64, tag = "4")]
    pub write_head: i64,
    /// Fragment to which the offset was mapped. This is a metadata field and will
    /// not be returned with a content response.
    #[prost(message, optional, tag = "5")]
    pub fragment: ::core::option::Option<Fragment>,
    /// If Fragment is remote, a URL from which it may be directly read.
    #[prost(string, tag = "6")]
    pub fragment_url: ::prost::alloc::string::String,
    /// Content chunks of the read.
    #[prost(bytes = "vec", tag = "7")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
/// AppendRequest is the streamed request message of the broker Append RPC.
/// Append request streams consist of an initial message having all parameters
/// of the append, such as the journal to append to and preconditions, followed
/// by an unbounded number of messages having only content "chunks".
///
/// It's not required that the appender know the append size when starting the
/// Append RPC -- rather, the client indicates the stream is complete by sending
/// a final, empty "chunk" message. However be aware that the broker will
/// aggressively time out stalled Append clients, and clients should not start
/// RPCs until all content chunks are available for immediate writing.
///
/// Append RPCs also expose a concept of journal "registers": LabelSets
/// which participate in the journal's transactional append machinery.
/// Note that registers are sent and verified with every replicated journal
/// transaction, so they're _really_ intended to be very small.
///
/// Append RPCs may upsert (union) or delete (subtract) labels from the
/// journal's registers. Register consensus is achieved by piggy-backing on the
/// append itself: if peers disagree, the registers of the replica having the
/// largest journal byte offset always win. For this reason, only RPCs appending
/// at least one byte may modify registers.
///
/// Append RPCs may also require that registers match an arbitrary selector
/// before the RPC may proceed. For example, a write fence can be implemented
/// by requiring that a "author" register is of an expected value. At-most-once
/// semantics can be implemented as a check-and-set over a single register.
///
/// Also be aware that a register update can still occur even for RPCs which are
/// reported as failed to the client. That's because an append RPC succeeds
/// only after all replicas acknowledge it, but a RPC which applies to some
/// replicas but not all still moves the journal offset forward, and therefore
/// updates journal registers.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendRequest {
    /// Header is attached by a proxying broker peer to the first AppendRequest
    /// message.
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    /// Journal to be appended to.
    #[prost(string, tag = "2")]
    pub journal: ::prost::alloc::string::String,
    /// If do_not_proxy is true, the broker will not proxy the append if it is
    /// not the current primary.
    #[prost(bool, tag = "3")]
    pub do_not_proxy: bool,
    /// Journal offset at which the append should begin. Most clients should leave
    /// at zero, which uses the broker's tracked offset. The append offset must be
    /// one greater than furthest written offset of the journal, or
    /// WRONG_APPEND_OFFSET is returned.
    #[prost(int64, tag = "5")]
    pub offset: i64,
    /// Selector of journal registers which must be satisfied for the request
    /// to proceed. If not matched, the RPC is failed with REGISTER_MISMATCH.
    ///
    /// There's one important exception: if the set of registers associated with
    /// a journal is completely empty, then *any* selector is considered as
    /// matching. While perhaps surprising, this behavior supports the intended
    /// use of registers for cooperative locking, whereby an empty set of
    /// registers can be thought of as an "unlocked" state. More practically, if
    /// Etcd consensus is lost then so are current register values: on recovery
    /// journals will restart with an empty set. This behavior ensures that an
    /// existing process holding a prior lock can continue to write -- at least
    /// until another process updates registers once again.
    #[prost(message, optional, tag = "6")]
    pub check_registers: ::core::option::Option<LabelSelector>,
    /// Labels to union with current registers if the RPC succeeds and appended
    /// at least one byte.
    #[prost(message, optional, tag = "7")]
    pub union_registers: ::core::option::Option<LabelSet>,
    /// Labels to subtract from current registers if the RPC succeeds and appended
    /// at least one byte.
    #[prost(message, optional, tag = "8")]
    pub subtract_registers: ::core::option::Option<LabelSet>,
    /// Content chunks to be appended. Immediately prior to closing the stream,
    /// the client must send an empty chunk (eg, zero-valued AppendRequest) to
    /// indicate the Append should be committed. Absence of this empty chunk
    /// prior to EOF is interpreted by the broker as a rollback of the Append.
    #[prost(bytes = "vec", tag = "4")]
    pub content: ::prost::alloc::vec::Vec<u8>,
}
/// AppendResponse is the unary response message of the broker Append RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendResponse {
    /// Status of the Append RPC.
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    /// Header of the response.
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<Header>,
    /// If status is OK, then |commit| is the Fragment which places the
    /// committed Append content within the Journal.
    #[prost(message, optional, tag = "3")]
    pub commit: ::core::option::Option<Fragment>,
    /// Current registers of the journal.
    #[prost(message, optional, tag = "4")]
    pub registers: ::core::option::Option<LabelSet>,
    /// Total number of RPC content chunks processed in this append.
    #[prost(int64, tag = "5")]
    pub total_chunks: i64,
    /// Number of content chunks which were delayed by journal flow control.
    #[prost(int64, tag = "6")]
    pub delayed_chunks: i64,
}
/// ReplicateRequest is the streamed request message of the broker's internal
/// Replicate RPC. Each message is either a pending content chunk or a
/// "proposal" to commit (or roll back) content chunks previously sent.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReplicateRequest {
    /// Header defines the primary broker, Route, and Etcd Revision under which
    /// this Replicate stream is being established. Each replication peer
    /// independently inspects and verifies the current Journal Route topology.
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    /// Proposed Fragment to commit, which is verified by each replica.
    #[prost(message, optional, tag = "3")]
    pub proposal: ::core::option::Option<Fragment>,
    /// Registers proposed to apply, which are also verified by each replica.
    #[prost(message, optional, tag = "7")]
    pub registers: ::core::option::Option<LabelSet>,
    /// Acknowledge requests that the peer send an acknowledging ReplicateResponse
    /// on successful application of the ReplicateRequest.
    #[prost(bool, tag = "6")]
    pub acknowledge: bool,
    /// Journal to be replicated to, which is also captured by |proposal|.
    /// Deprecated.
    #[prost(string, tag = "2")]
    pub deprecated_journal: ::prost::alloc::string::String,
    /// Content to be replicated.
    #[prost(bytes = "vec", tag = "4")]
    pub content: ::prost::alloc::vec::Vec<u8>,
    /// Delta offset of |content| relative to current Fragment |end|.
    #[prost(int64, tag = "5")]
    pub content_delta: i64,
}
/// ReplicateResponse is the streamed response message of the broker's internal
/// Replicate RPC. Each message is a 1:1 response to a previously read "proposal"
/// ReplicateRequest with |acknowledge| set.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReplicateResponse {
    /// Status of the Replicate RPC.
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    /// Header of the response. Accompanies the first ReplicateResponse of the
    /// response stream.
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<Header>,
    /// If status is PROPOSAL_MISMATCH, then |fragment| is the replica's current
    /// journal Fragment, and either it or |registers| will differ from the
    /// primary's proposal.
    #[prost(message, optional, tag = "3")]
    pub fragment: ::core::option::Option<Fragment>,
    /// If status is PROPOSAL_MISMATCH, then |registers| are the replica's current
    /// journal registers.
    #[prost(message, optional, tag = "4")]
    pub registers: ::core::option::Option<LabelSet>,
}
/// ListRequest is the unary request message of the broker List RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRequest {
    /// Selector optionally refines the set of journals which will be enumerated.
    /// If zero-valued, all journals are returned. Otherwise, only JournalSpecs
    /// matching the LabelSelector will be returned. Two meta-labels "name" and
    /// "prefix" are additionally supported by the selector, where:
    ///   * name=examples/a-name will match a JournalSpec with Name
    ///   "examples/a-name"
    ///   * prefix=examples/ will match any JournalSpec having prefix "examples/".
    ///     The prefix Label value must end in '/'.
    #[prost(message, optional, tag = "1")]
    pub selector: ::core::option::Option<LabelSelector>,
}
/// ListResponse is the unary response message of the broker List RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResponse {
    /// Status of the List RPC.
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    /// Header of the response.
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, repeated, tag = "3")]
    pub journals: ::prost::alloc::vec::Vec<list_response::Journal>,
}
/// Nested message and enum types in `ListResponse`.
pub mod list_response {
    /// Journals of the response.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Journal {
        #[prost(message, optional, tag = "1")]
        pub spec: ::core::option::Option<super::JournalSpec>,
        /// Current ModRevision of the JournalSpec.
        #[prost(int64, tag = "2")]
        pub mod_revision: i64,
        /// Route of the journal, including endpoints.
        #[prost(message, optional, tag = "3")]
        pub route: ::core::option::Option<super::Route>,
    }
}
/// ApplyRequest is the unary request message of the broker Apply RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ApplyRequest {
    #[prost(message, repeated, tag = "1")]
    pub changes: ::prost::alloc::vec::Vec<apply_request::Change>,
}
/// Nested message and enum types in `ApplyRequest`.
pub mod apply_request {
    /// Change defines an insertion, update, or deletion to be applied to the set
    /// of JournalSpecs. Exactly one of |upsert| or |delete| must be set.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Change {
        /// Expected ModRevision of the current JournalSpec. If the Journal is being
        /// created, expect_mod_revision is zero.
        #[prost(int64, tag = "1")]
        pub expect_mod_revision: i64,
        /// JournalSpec to be updated (if expect_mod_revision > 0) or created
        /// (if expect_mod_revision == 0).
        #[prost(message, optional, tag = "2")]
        pub upsert: ::core::option::Option<super::JournalSpec>,
        /// Journal to be deleted. expect_mod_revision must not be zero.
        #[prost(string, tag = "3")]
        pub delete: ::prost::alloc::string::String,
    }
}
/// ApplyResponse is the unary response message of the broker Apply RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ApplyResponse {
    /// Status of the Apply RPC.
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    /// Header of the response.
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<Header>,
}
/// FragmentsRequest is the unary request message of the broker ListFragments
/// RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FragmentsRequest {
    /// Header is attached by a proxying broker peer.
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    /// Journal to be read.
    #[prost(string, tag = "2")]
    pub journal: ::prost::alloc::string::String,
    /// BeginModTime is an optional field specifying an inclusive lower bound on
    /// the modification timestamp for a fragment to be returned. The timestamp is
    /// represented as seconds since the epoch.
    #[prost(int64, tag = "3")]
    pub begin_mod_time: i64,
    /// EndModTime is an optional field specifying an exclusive upper bound on
    /// the modification timestamp for a fragment to be returned. The timestamp is
    /// represented as seconds since the epoch.
    #[prost(int64, tag = "4")]
    pub end_mod_time: i64,
    /// The NextPageToke value returned from a previous, continued
    /// FragmentsRequest, if any.
    #[prost(int64, tag = "5")]
    pub next_page_token: i64,
    /// PageLimit is an optional field specifying how many fragments to return
    /// with the response. The default value for PageLimit is 1000.
    #[prost(int32, tag = "6")]
    pub page_limit: i32,
    /// SignatureTTL indicates that a temporary signed GET URL should be returned
    /// with each response Fragment, valid for |signatureTTL|.
    #[prost(message, optional, tag = "7")]
    pub signature_ttl: ::core::option::Option<::prost_types::Duration>,
    /// If do_not_proxy is true, the broker will not proxy the request to another
    /// broker on the client's behalf.
    #[prost(bool, tag = "8")]
    pub do_not_proxy: bool,
}
/// FragmentsResponse is the unary response message of the broker ListFragments
/// RPC.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FragmentsResponse {
    /// Status of the Apply RPC.
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
    /// Header of the response.
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, repeated, tag = "3")]
    pub fragments: ::prost::alloc::vec::Vec<fragments_response::Fragment>,
    /// The NextPageToke value to be returned on subsequent Fragments requests. If
    /// the value is zero then there are no more fragments to be returned for this
    /// page.
    #[prost(int64, tag = "4")]
    pub next_page_token: i64,
}
/// Nested message and enum types in `FragmentsResponse`.
pub mod fragments_response {
    /// Fragments of the Response.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Fragment {
        #[prost(message, optional, tag = "1")]
        pub spec: ::core::option::Option<super::Fragment>,
        /// SignedURL is a temporary URL at which a direct GET of the Fragment may
        /// be issued, signed by the broker's credentials. Set only if the request
        /// specified a SignatureTTL.
        #[prost(string, tag = "2")]
        pub signed_url: ::prost::alloc::string::String,
    }
}
/// Route captures the current topology of an item and the processes serving it.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Route {
    /// Members of the Route, ordered on ascending ProcessSpec.ID (zone, suffix).
    #[prost(message, repeated, tag = "1")]
    pub members: ::prost::alloc::vec::Vec<process_spec::Id>,
    /// Index of the ProcessSpec serving as primary within |members|,
    /// or -1 of no member is currently primary.
    #[prost(int32, tag = "2")]
    pub primary: i32,
    /// Endpoints of each Route member. If not empty, |endpoints| has the same
    /// length and order as |members|, and captures the endpoint of each one.
    #[prost(string, repeated, tag = "3")]
    pub endpoints: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Header captures metadata such as the process responsible for processing
/// an RPC, and its effective Etcd state.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// ID of the process responsible for request processing. May be empty iff
    /// Header is being used within a proxied request, and that request may be
    /// dispatched to any member of the Route.
    #[prost(message, optional, tag = "1")]
    pub process_id: ::core::option::Option<process_spec::Id>,
    /// Route of processes specifically responsible for this RPC, or an empty Route
    /// if any process is capable of serving the RPC.
    #[prost(message, optional, tag = "2")]
    pub route: ::core::option::Option<Route>,
    #[prost(message, optional, tag = "3")]
    pub etcd: ::core::option::Option<header::Etcd>,
}
/// Nested message and enum types in `Header`.
pub mod header {
    /// Etcd represents the effective Etcd MVCC state under which a Gazette broker
    /// is operating in its processing of requests and responses. Its inclusion
    /// allows brokers to reason about relative "happened before" Revision ordering
    /// of apparent routing conflicts in proxied or replicated requests, as well
    /// as enabling sanity checks over equality of Etcd ClusterId (and precluding,
    /// for example, split-brain scenarios where different brokers are backed by
    /// different Etcd clusters). Etcd is kept in sync with
    /// etcdserverpb.ResponseHeader.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Etcd {
        /// cluster_id is the ID of the cluster.
        #[prost(uint64, tag = "1")]
        pub cluster_id: u64,
        /// member_id is the ID of the member.
        #[prost(uint64, tag = "2")]
        pub member_id: u64,
        /// revision is the Etcd key-value store revision when the request was
        /// applied.
        #[prost(int64, tag = "3")]
        pub revision: i64,
        /// raft_term is the raft term when the request was applied.
        #[prost(uint64, tag = "4")]
        pub raft_term: u64,
    }
}
/// Status is a response status code, used universally across Gazette RPC APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    /// The named journal does not exist.
    JournalNotFound = 1,
    /// There is no current primary broker for the journal. This is a temporary
    /// condition which should quickly resolve, assuming sufficient broker
    /// capacity.
    NoJournalPrimaryBroker = 2,
    /// The present broker is not the assigned primary broker for the journal.
    NotJournalPrimaryBroker = 3,
    /// The present broker is not an assigned broker for the journal.
    NotJournalBroker = 5,
    /// There are an insufficient number of assigned brokers for the journal
    /// to meet its required replication.
    InsufficientJournalBrokers = 4,
    /// The requested offset is not yet available. This indicates either that the
    /// offset has not yet been written, or that the broker is not yet aware of a
    /// written fragment covering the offset. Returned only by non-blocking reads.
    OffsetNotYetAvailable = 6,
    /// The peer disagrees with the Route accompanying a ReplicateRequest.
    WrongRoute = 7,
    /// The peer disagrees with the proposal accompanying a ReplicateRequest.
    ProposalMismatch = 8,
    /// The Etcd transaction failed. Returned by Update RPC when an
    /// expect_mod_revision of the UpdateRequest differs from the current
    /// ModRevision of the JournalSpec within the store.
    EtcdTransactionFailed = 9,
    /// A disallowed journal access was attempted (eg, a write where the
    /// journal disables writes, or read where journals disable reads).
    NotAllowed = 10,
    /// The Append is refused because its requested offset is not equal
    /// to the furthest written offset of the journal.
    WrongAppendOffset = 11,
    /// The Append is refused because the replication pipeline tracks a smaller
    /// journal offset than that of the remote fragment index. This indicates
    /// that journal replication consistency has been lost in the past, due to
    /// too many broker or Etcd failures.
    IndexHasGreaterOffset = 12,
    /// The Append is refused because a registers selector was provided with the
    /// request, but it was not matched by current register values of the journal.
    RegisterMismatch = 13,
}
/// CompressionCode defines codecs known to Gazette.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CompressionCodec {
    /// INVALID is the zero-valued CompressionCodec, and is not a valid codec.
    Invalid = 0,
    /// NONE encodes Fragments without any applied compression, with default suffix
    /// ".raw".
    None = 1,
    /// GZIP encodes Fragments using the Gzip library, with default suffix ".gz".
    Gzip = 2,
    /// ZSTANDARD encodes Fragments using the ZStandard library, with default
    /// suffix ".zst".
    Zstandard = 3,
    /// SNAPPY encodes Fragments using the Snappy library, with default suffix
    /// ".sz".
    Snappy = 4,
    /// GZIP_OFFLOAD_DECOMPRESSION is the GZIP codec with additional behavior
    /// around reads and writes to remote Fragment stores, designed to offload
    /// the work of decompression onto compatible stores. Specifically:
    ///  * Fragments are written with a "Content-Encoding: gzip" header.
    ///  * Client read requests are made with "Accept-Encoding: identity".
    /// This can be helpful in contexts where reader IO bandwidth to the storage
    /// API is unconstrained, as the cost of decompression is offloaded to the
    /// store and CPU-intensive batch readers may receive a parallelism benefit.
    /// While this codec may provide substantial read-time performance
    /// improvements, it is an advanced configuration and the "Content-Encoding"
    /// header handling can be subtle and sometimes confusing. It uses the default
    /// suffix ".gzod".
    GzipOffloadDecompression = 5,
}
#[doc = r" Generated client implementations."]
#[cfg(feature = "gaz_broker_client")]
pub mod journal_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = " Journal is the Gazette broker service API for interacting with Journals."]
    #[derive(Debug, Clone)]
    pub struct JournalClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl JournalClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> JournalClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> JournalClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            JournalClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " List Journals, their JournalSpecs and current Routes."]
        pub async fn list(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRequest>,
        ) -> Result<tonic::Response<super::ListResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/protocol.Journal/List");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Apply changes to the collection of Journals managed by the brokers."]
        pub async fn apply(
            &mut self,
            request: impl tonic::IntoRequest<super::ApplyRequest>,
        ) -> Result<tonic::Response<super::ApplyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/protocol.Journal/Apply");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Read from a specific Journal."]
        pub async fn read(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadRequest>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::ReadResponse>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/protocol.Journal/Read");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        #[doc = " Append content to a specific Journal."]
        pub async fn append(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::AppendRequest>,
        ) -> Result<tonic::Response<super::AppendResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/protocol.Journal/Append");
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
        #[doc = " Replicate appended content of a Journal. Replicate is used between broker"]
        #[doc = " peers in the course of processing Append transactions, but is not intended"]
        #[doc = " for direct use by clients."]
        pub async fn replicate(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::ReplicateRequest>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::ReplicateResponse>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/protocol.Journal/Replicate");
            self.inner
                .streaming(request.into_streaming_request(), path, codec)
                .await
        }
        #[doc = " List Fragments of a Journal."]
        pub async fn list_fragments(
            &mut self,
            request: impl tonic::IntoRequest<super::FragmentsRequest>,
        ) -> Result<tonic::Response<super::FragmentsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/protocol.Journal/ListFragments");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
#[cfg(feature = "gaz_broker_server")]
pub mod journal_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with JournalServer."]
    #[async_trait]
    pub trait Journal: Send + Sync + 'static {
        #[doc = " List Journals, their JournalSpecs and current Routes."]
        async fn list(
            &self,
            request: tonic::Request<super::ListRequest>,
        ) -> Result<tonic::Response<super::ListResponse>, tonic::Status>;
        #[doc = " Apply changes to the collection of Journals managed by the brokers."]
        async fn apply(
            &self,
            request: tonic::Request<super::ApplyRequest>,
        ) -> Result<tonic::Response<super::ApplyResponse>, tonic::Status>;
        #[doc = "Server streaming response type for the Read method."]
        type ReadStream: futures_core::Stream<Item = Result<super::ReadResponse, tonic::Status>>
            + Send
            + 'static;
        #[doc = " Read from a specific Journal."]
        async fn read(
            &self,
            request: tonic::Request<super::ReadRequest>,
        ) -> Result<tonic::Response<Self::ReadStream>, tonic::Status>;
        #[doc = " Append content to a specific Journal."]
        async fn append(
            &self,
            request: tonic::Request<tonic::Streaming<super::AppendRequest>>,
        ) -> Result<tonic::Response<super::AppendResponse>, tonic::Status>;
        #[doc = "Server streaming response type for the Replicate method."]
        type ReplicateStream: futures_core::Stream<Item = Result<super::ReplicateResponse, tonic::Status>>
            + Send
            + 'static;
        #[doc = " Replicate appended content of a Journal. Replicate is used between broker"]
        #[doc = " peers in the course of processing Append transactions, but is not intended"]
        #[doc = " for direct use by clients."]
        async fn replicate(
            &self,
            request: tonic::Request<tonic::Streaming<super::ReplicateRequest>>,
        ) -> Result<tonic::Response<Self::ReplicateStream>, tonic::Status>;
        #[doc = " List Fragments of a Journal."]
        async fn list_fragments(
            &self,
            request: tonic::Request<super::FragmentsRequest>,
        ) -> Result<tonic::Response<super::FragmentsResponse>, tonic::Status>;
    }
    #[doc = " Journal is the Gazette broker service API for interacting with Journals."]
    #[derive(Debug)]
    pub struct JournalServer<T: Journal> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Journal> JournalServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for JournalServer<T>
    where
        T: Journal,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/protocol.Journal/List" => {
                    #[allow(non_camel_case_types)]
                    struct ListSvc<T: Journal>(pub Arc<T>);
                    impl<T: Journal> tonic::server::UnaryService<super::ListRequest> for ListSvc<T> {
                        type Response = super::ListResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protocol.Journal/Apply" => {
                    #[allow(non_camel_case_types)]
                    struct ApplySvc<T: Journal>(pub Arc<T>);
                    impl<T: Journal> tonic::server::UnaryService<super::ApplyRequest> for ApplySvc<T> {
                        type Response = super::ApplyResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ApplyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).apply(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ApplySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protocol.Journal/Read" => {
                    #[allow(non_camel_case_types)]
                    struct ReadSvc<T: Journal>(pub Arc<T>);
                    impl<T: Journal> tonic::server::ServerStreamingService<super::ReadRequest> for ReadSvc<T> {
                        type Response = super::ReadResponse;
                        type ResponseStream = T::ReadStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).read(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReadSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protocol.Journal/Append" => {
                    #[allow(non_camel_case_types)]
                    struct AppendSvc<T: Journal>(pub Arc<T>);
                    impl<T: Journal> tonic::server::ClientStreamingService<super::AppendRequest> for AppendSvc<T> {
                        type Response = super::AppendResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::AppendRequest>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).append(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppendSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.client_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protocol.Journal/Replicate" => {
                    #[allow(non_camel_case_types)]
                    struct ReplicateSvc<T: Journal>(pub Arc<T>);
                    impl<T: Journal> tonic::server::StreamingService<super::ReplicateRequest> for ReplicateSvc<T> {
                        type Response = super::ReplicateResponse;
                        type ResponseStream = T::ReplicateStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::ReplicateRequest>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).replicate(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReplicateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protocol.Journal/ListFragments" => {
                    #[allow(non_camel_case_types)]
                    struct ListFragmentsSvc<T: Journal>(pub Arc<T>);
                    impl<T: Journal> tonic::server::UnaryService<super::FragmentsRequest> for ListFragmentsSvc<T> {
                        type Response = super::FragmentsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FragmentsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_fragments(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListFragmentsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Journal> Clone for JournalServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Journal> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Journal> tonic::transport::NamedService for JournalServer<T> {
        const NAME: &'static str = "protocol.Journal";
    }
}
