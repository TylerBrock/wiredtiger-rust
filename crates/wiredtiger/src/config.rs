pub struct OpenConnectionConfig {
    // in-memory alignment (in bytes) for buffers used for I/O.
    // The default value of -1 indicates a platform-specific alignment value should be used
    // (4KB on Linux systems, zero elsewhere). An integer between -1 and 1MB; default -1.
    buffer_alignment: i32,

    // Assume the heap allocator overhead is the specified percentage,
    // and adjust the cache usage by that amount (for example, if there is 10GB of data in cache,
    // a percentage of 10 means WiredTiger treats this as 11GB).
    // This value is configurable because different heap allocators have different overhead and
    // different workloads will have different heap allocation sizes and patterns,
    // therefore applications may need to adjust this value based on allocator choice and behavior in measured workloads.
    // An integer between 0 and 30; default 8.
    cache_overhead: u8,

    // Maximum heap memory to allocate for the cache.
    // A database should configure either cache_size or shared_cache but not both.
    // An integer between 1MB and 10TB; default 100MB.
    cache_size: u32,

    checkpoint: CheckpointConfig,

    // Flush files to stable storage when closing or writing checkpoints. Default true.
    checkpoint_sync: bool,

    // Write the base configuration file if creating the database, see WiredTiger.basecfg file for more information.
    // Default true.
    config_base: bool,

    // Create the database if it does not exist. Default false.
    create: bool,

    // Use O_DIRECT to access files. Options are given as a list, such as "direct_io=[data]".
    // Configuring direct_io requires care, see Direct I/O for important warnings.
    // Including "data" will cause WiredTiger data files to use O_DIRECT,
    // including "log" will cause WiredTiger log files to use O_DIRECT,
    // and including "checkpoint" will cause WiredTiger data files opened at a checkpoint (i.e: read only) to use O_DIRECT.
    // list, with values chosen from the following options: "checkpoint", "data", "log"; default empty.
    direct_io: Vec<DirectIOSetting>,

    // Prefix string for error messages. Default empty.
    error_prefix: String,

    eviction: EvictionConfig,

    // Continue evicting until the cache has less dirty memory than the value, as a percentage of the total cache size.
    // Dirty pages will only be evicted if the cache is full enough to trigger eviction. An integer between 10 and 99; default 80.
    eviction_dirty_target: i8,

    // Continue evicting until the cache has less total memory than the value, as a percentage of the total cache size.
    // Must be less than eviction_trigger. An integer between 10 and 99; default 80.
    eviction_target: i8,

    // Trigger eviction when the cache is using this much memory,
    // as a percentage of the total cache size.
    // An integer between 10 and 99; default 95.
    eviction_trigger: i8,

    // Fail if the database already exists, generally used with the create option. Default false.
    exclusive: bool,

    // list of shared library extensions to load (using dlopen).
    // Any values specified to an library extension are passed to
    // WT_CONNECTION::load_extension as the config parameter (for example, extensions=(/path/ext.so={entry=my_entry})).
    // A list of strings; default empty.
    extensions: Vec<String>,

    // File extension configuration. If set, extend files of the set type
    // in allocations of the set size, instead of a block at a time as each
    // new block is written. For example, file_extend=(data=16MB).
    // A list, with values chosen from the following options: "data", "log"; default empty.
    file_extend: Vec<FileExtensionConfigOption>,

    // Maximum number of simultaneous hazard pointers per session handle.
    // An integer greater than or equal to 15; default 1000.
    hazard_max: i16,

    log: LogConfig,

    shared_cache: SharedCacheConfig,

    // Maintain database statistics, which may impact performance.
    // Choosing "all" maintains all statistics regardless of cost,
    // "fast" maintains a subset of statistics that are relatively inexpensive,
    // "none" turns off all statistics.
    // The "clear" configuration resets statistics after they are gathered,
    // where appropriate (for example, a cache size statistic is not cleared,
    // while the count of cursor insert operations will be cleared).
    // When "clear" is configured for the database, gathered statistics are reset
    // each time a statistics cursor is used to gather statistics, as well as each time
    // statistics are logged using the statistics_log configuration.
    //  See Statistics for more information.
    // A list, with values chosen from the following options: "all", "fast", "none", "clear"; default none.
    statistics: Vec<StatisticsOption>,

    statistics_log: StatisticsLogConfig,

    transaction_sync: TransactionSyncConfig,

    // Use the WIREDTIGER_CONFIG and WIREDTIGER_HOME environment variables
    // regardless of whether or not the process is running with special privileges.
    // See Database Home Directory for more information. A boolean flag; default false.
    use_environment_priv: bool,

    // Enable messages for various events.
    // Only available if WiredTiger is configured with –enable-verbose.
    // Options are given as a list, such as "verbose=[evictserver,read]".
    // A list, with values chosen from the following options:
    // "api", "block", "checkpoint", "compact", "evict", "evictserver",
    // "fileops", "log", "lsm", "metadata", "mutex", "overflow", "read",
    // "reconcile", "recovery", "salvage", "shared_cache", "split",
    // "temporary", "transaction", "verify", "version", "write".
    // Default empty.
    verbose: Vec<VerboseOption>,
}

impl OpenConnectionConfig {
    pub fn to_string(&self) -> String {
        "".to_string()
    }
}

enum VerboseOption {
    Api,
    Block,
    Checkpoint,
    Compact,
    Evict,
    EvictServer,
    FileOps,
    Log,
    Lsm,
    Metadata,
    Mutex,
    Overflow,
    Read,
    Reconcile,
    Recovery,
    Salvage,
    SharedCache,
    Split,
    Temporary,
    Transaction,
    Verify,
    Version,
    Write,
}

// How to sync log records when the transaction commits.
struct TransactionSyncConfig {
    //  Whether to sync the log on every commit by default,
    // can be overridden by the sync setting to WT_SESSION::begin_transaction.
    // A boolean flag; default false.
    enabled: bool,

    // The method used to ensure log records are stable on disk,
    // see Commit-level durability for more information.
    // A string, chosen from the following options: "dsync", "fsync", "none"; default fsync.
    method: SyncMethodOption,
}

enum SyncMethodOption {
    DSync,
    FSync,
    None,
}

struct StatisticsLogConfig {
    // log statistics on database close.	a boolean flag; default false.
    on_close: bool,

    // The pathname to a file into which the log records are written,
    // may contain ISO C standard strftime conversion specifications.
    // If the value is not an absolute path name, the file is created
    // relative to the database home. A string; default "WiredTigerStat.%d.%H".
    path: String,

    // If non-empty, include statistics for the list of data source URIs,
    // if they are open at the time of the statistics logging.
    // The list may include URIs matching a single data source ("table:mytable"),
    // or a URI matching all data sources of a particular type ("table:").
    // A list of strings; default empty.
    sources: Vec<String>,

    // a timestamp prepended to each log record, may contain strftime conversion specifications.	a string; default "%b %d %H:%M:%S".
    timestamp: String,

    // seconds to wait between each write of the log records; setting this value above 0 configures statistics logging.	an integer between 0 and 100000; default 0.
    wait: u16,
}

enum StatisticsOption {
    All,
    Fast,
    None,
    Clear,
}

struct LogConfig {
    // Automatically archive unneeded log files. Default true.
    archive: bool,

    // Configure a compressor for log records.
    // Permitted values are "none" or "bzip2", "snappy" or custom compression engine "name"
    // created with WT_CONNECTION::add_compressor. See Compressors for more information.
    // a string; default none.
    compressor: String, // TODO enum?

    // Enable logging subsystem. Default false.
    enabled: bool,

    // The maximum size of log files. An integer between 100KB and 2GB; default 100MB.
    file_max: i32,

    // The path to a directory into which the log files are written.
    // If the value is not an absolute path name, the files are created relative to the database home.
    // Default empty.
    path: String,

    // pre-allocate log files.	a boolean flag; default true.
    prealloc: bool,

    // Run recovery or error if recovery needs to run after an unclean shutdown.
    // A string, chosen from the following options: "error", "on"; default on.
    recover: String, // todo enum?

    // Use memory mapping to access files when possible. Default true.
    mmap: bool,

    // Permit sharing between processes (will automatically start an RPC server
    // for primary processes and use RPC for secondary processes).
    // Not yet supported in WiredTiger. A boolean flag; default false.
    multiprocess: bool,

    // Maximum expected number of sessions (including server threads).
    // An integer greater than or equal to 1; default 100.
    session_max: u16,
}

struct SharedCacheConfig {
    // The granularity that a shared cache is redistributed.
    // An integer between 1MB and 10TB; default 10MB.
    chunk: u32,

    // The name of a cache that is shared between databases or "none" when no shared cache is configured.
    // Default none.
    name: String,

    // Amount of cache this database is guaranteed to have available from the shared cache.
    // This setting is per database. Defaults to the chunk size. Default 0.
    reserve: u32,

    // Maximum memory to allocate for the shared cache.
    // Setting this will update the value if one is already set.
    // An integer between 1MB and 10TB; default 500MB.
    size: u32,
}

enum FileExtensionConfigOption {
    Data,
    Log,
}

struct EvictionConfig {
    // maximum number of threads WiredTiger will start to help evict pages from cache.
    // The number of threads started will vary depending on the current eviction load.
    // An integer between 1 and 20; default 1.
    threads_max: u8,
    // minimum number of threads WiredTiger will start to help evict pages from cache.
    // The number of threads currently running will vary depending on the current eviction load.
    // An integer between 1 and 20; default 1.
    threads_min: u8,
}

enum DirectIOSetting {
    Checkpoint,
    Data,
    Log,
}

struct CheckpointConfig {
    // Wait for this amount of log record bytes to be written to the log between each checkpoint.
    // A database can configure both log_size and wait to set an upper bound for checkpoints;
    // Setting this value above 0 configures periodic checkpoints.	An integer between 0 and 2GB; default 0.
    log_size: i32,

    // The checkpoint name. Default "WiredTigerCheckpoint".
    name: String,

    // Seconds to wait between each checkpoint; setting this value above 0 configures periodic checkpoints.
    // An integer between 0 and 100000; default 0.
    wait: i16,
}

struct AsyncConfig {
    // Enable asynchronous operation.	a boolean flag; default false.
    enabled: bool,

    // Maximum number of expected simultaneous asynchronous operations.
    // An integer between 10 and 4096; default 1024.
    ops_max: u16,

    // The number of worker threads to service asynchronous requests.
    // An integer between 1 and 20; default 2.
    threads: u8,
}

struct LSMManagerConfig {
    // Merge LSM chunks where possible. Default true.
    merge: bool,

    // Configure a set of threads to manage merging LSM trees in the database.
    // An integer between 3 and 20; default 4.
    worker_thread_max: u8,
}

struct OpenSessionConfig {
    // The default isolation level for operations in this session.
    // A string, chosen from the following options:
    //  "read-uncommitted", "read-committed", "snapshot";
    // Default read-committed.
    isolation: IsolationLevel,
}

enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    Snapshot,
}

enum BlockAllocationOption {
    First,
    Best,
}

enum ChecksumOption {
    On,
    Off,
    Uncompressed,
}

struct CreateConfig {
    // The file unit allocation size, in bytes, must a power-of-two; smaller values decrease the file space required by overflow items, and the default value of 4KB is a good choice absent requirements from the operating system or storage device.	an integer between 512B and 128MB; default 4KB.
    allocation_size: u32,

    // Application-owned metadata for this object.	a string; default empty.
    app_metadata: String,

    // Configure block allocation. Permitted values are "first" or "best";
    // the "first" configuration uses a first-available algorithm during block allocation,
    // the "best" configuration uses a best-fit algorithm. Default "best".
    block_allocation: BlockAllocationOption,

    // Configure a compressor for file blocks. Permitted values are "none" or custom compression engine name created with WT_CONNECTION::add_compressor.
    // If WiredTiger has builtin support for "snappy" or "zlib" compression, these names are also available. See Compressors for more information.	a string; default none.
    block_compressor: String, // TODO enum?

    // Do not ever evict the object's pages; see Cache resident objects for more information.	a boolean flag; default false.
    cache_resident: bool,

    // Configure block checksums; permitted values are on (checksum all blocks),
    //  off (checksum no blocks) and uncompresssed (checksum only blocks which are not compressed for any reason).
    //  The uncompressed setting is for applications which can rely on decompression to fail if a block has been corrupted.
    //	A string, chosen from the following options: "on", "off", "uncompressed"; default uncompressed.
    checksum: ChecksumOption,

    // Comma-separated list of names of column groups.
    //  Each column group is stored separately, keyed by the primary key of the table.
    // If no column groups are specified, all columns are stored together in a single file.
    // All value columns in the table must appear in at least one column group.
    //  Each column group must be created with a separate call to WT_SESSION::create. A list of strings; default empty.
    colgroups: Vec<String>,

    // Configure custom collation for keys. Permitted values are "none" or a custom collator name created with WT_CONNECTION::add_collator.	A string; default none.
    collator: String,

    // List of the column names. Comma-separated list of the form (column[,...]).
    // For tables, the number of entries must match the total number of values in key_format and value_format.
    // For colgroups and indices, all column names must appear in the list of columns for the table.
    // A list of strings; default empty.
    columns: Vec<String>,

    // The maximum number of unique values remembered in the Btree row-store leaf page value dictionary;
    // see File formats and compression for more information.
    // An integer greater than or equal to 0; default 0.
    dictionary: u32,

    // Fail if the object exists. When false (the default), if the object exists, check that its settings match the specified configuration.
    // A boolean flag; default false.
    exclusive: bool,

    // Configure custom extractor for indices. Permitted values are "none" or an extractor name created with WT_CONNECTION::add_extractor.	a string; default none.
    extractor: String, // TODO enum?

    // The file format.	a string, chosen from the following options: "btree"; default btree.
    format: String, // TODO enum?

    // Configure Huffman encoding for keys. Permitted values are "none", "english", "utf8<file>" or "utf16<file>". See Huffman Encoding for more information.	a string; default none.
    huffman_key: String, // TODO

    // Configure Huffman encoding for values. Permitted values are "none", "english", "utf8<file>" or "utf16<file>". See Huffman Encoding for more information.	a string; default none.
    huffman_value: String, // TODO

    // Configure the index to be immutable - that is an index is not changed by any update to a record in the table.	a boolean flag; default false.
    immutable: bool,

    // The largest key stored in an internal node, in bytes.
    //  If set, keys larger than the specified size are stored as overflow items (which may require additional I/O to access).
    //  The default and the maximum allowed value are both one-tenth the size of a newly split internal page.
    //	An integer greater than or equal to 0; default 0.
    internal_key_max: u16,

    // Configure internal key truncation, discarding unnecessary trailing bytes on internal keys (ignored for custom collators).	a boolean flag; default true.
    internal_key_truncate: bool,

    // The maximum page size for internal nodes, in bytes;
    //  the size must be a multiple of the allocation size and is significant for applications wanting to avoid excessive L2 cache misses while searching the tree.
    // The page maximum is the bytes of uncompressed data, that is, the limit is applied before any block compression is done.
    // An integer between 512B and 512MB; default 4KB.
    internal_page_max: u16,

    // The format of the data packed into key items.
    //  See Format types for details. By default, the key_format is 'u' and applications use WT_ITEM structures to manipulate raw byte arrays.
    // By default, records are stored in row-store files: keys of type 'r' are record numbers and records referenced by record number are stored in column-store files.
    // A format string; default u.
    key_format: String,

    // The largest key stored in a leaf node, in bytes.
    // If set, keys larger than the specified size are stored as overflow items
    //(which may require additional I/O to access).
    // The default value is one-tenth the size of a newly split leaf page.
    // An integer greater than or equal to 0; default 0.
    leaf_key_max: u16,

    // The maximum page size for leaf nodes, in bytes;
    // the size must be a multiple of the allocation size,
    // and is significant for applications wanting to maximize
    // sequential data transfer from a storage device.
    // The page maximum is the bytes of uncompressed data, that is,
    //  the limit is applied before any block compression is done. An integer between 512B and 512MB; default 32KB.
    leaf_page_max: u16,

    // The largest value stored in a leaf node, in bytes.
    // If set, values larger than the specified size are stored as
    // overflow items (which may require additional I/O to access).
    // If the size is larger than the maximum leaf page size,
    // the page size is temporarily ignored when large values are written.
    // The default is one-half the size of a newly split leaf page.
    // An integer greater than or equal to 0; default 0.
    leaf_value_max: u16,

    lsm_config: LSMConfig,

    // The maximum size a page can grow to in memory before being reconciled to disk.
    // The specified size will be adjusted to a lower bound of 50 * leaf_page_max,
    //  and an upper bound of cache_size / 2.
    //  This limit is soft - it is possible for pages to be temporarily larger than this value.
    //  This setting is ignored for LSM trees, see chunk_size.
    // An integer between 512B and 10TB; default 5MB.
    memory_page_max: u32,

    // Maximum dirty system buffer cache usage, in bytes.
    //  If non-zero, schedule writes for dirty blocks belonging to this
    // object in the system buffer cache after that many bytes from this
    // object are written into the buffer cache.
    // An integer greater than or equal to 0; default 0.
    os_cache_dirty_max: u32,

    // Maximum system buffer cache usage, in bytes.
    // If non-zero, evict object blocks from the system buffer
    // cache after that many bytes from this object are read or
    // written into the buffer cache.
    // An integer greater than or equal to 0; default 0.
    os_cache_max: u32,

    // Configure prefix compression on row-store leaf pages.
    // A boolean flag; default false.
    prefix_compression: bool,

    // Minimum gain before prefix compression will be used on row-store leaf pages.
    // An integer greater than or equal to 0; default 4.
    prefix_compression_min: u16,

    // The Btree page split size as a percentage of the maximum Btree page size,
    //  that is, when a Btree page is split, it will be split into smaller pages,
    //  where each page is the specified percentage of the maximum Btree page size.
    //	An integer between 25 and 100; default 75.
    split_pct: u16,

    // Set the type of data source used to store a column group, index or simple table.
    // By default, a "file:" URI is derived from the object name.
    //  The type configuration can be used to switch to a different data source,
    //  such as LSM or an extension configured by the application.
    // A string; default file.
    data_type: String,

    // The format of the data packed into value items. See Format types for details.
    //  By default, the value_format is 'u' and applications use a WT_ITEM structure to manipulate raw byte arrays.
    //  Value items of type 't' are bitfields, and when configured with record number type keys,
    //  will be stored using a fixed-length store.
    // A format string; default u.
    value_format: String,
}

struct DropConfig {
    // return success if the object does not exist.	Default false.
    force: bool,

    // should the underlying files be removed? Default true.
    remove_files: bool,
}

struct LSMConfig {
    // Throttle inserts into LSM trees if flushing to disk isn't keeping up.
    // A boolean flag; default true.
    auto_throttle: bool,

    // Create bloom filters on LSM tree chunks as they are merged.
    // A boolean flag; default true.
    bloom: bool,

    // The number of bits used per item for LSM bloom filters.
    // An integer between 2 and 1000; default 16.
    bloom_bit_count: u16,

    // Config string used when creating Bloom filter files, passed to WT_SESSION::create.
    // A string; default empty.
    bloom_config: String,

    // The number of hash values per item used for LSM bloom filters.
    // An integer between 2 and 100; default 8.
    bloom_hash_count: i16,

    // Create a bloom filter on the oldest LSM tree chunk.
    // Only supported if bloom filters are enabled.
    // A boolean flag; default false.
    bloom_oldest: bool,

    // The maximum number of chunks to allow in an LSM tree.
    // This option automatically times out old data.
    // As new chunks are added old chunks will be removed.
    // Enabling this option disables LSM background merges.
    // An integer; default 0.
    chunk_count_limit: u32,

    // The maximum size a single chunk can be.
    // Chunks larger than this size are not considered for further merges.
    // This is a soft limit, and chunks larger than this value can be created.
    // Must be larger than chunk_size.
    // An integer between 100MB and 10TB; default 5GB.
    chunk_max: u32,

    // The maximum size of the in-memory chunk of an LSM tree.
    // This limit is soft - it is possible for chunks to be temporarily larger than this value.
    // This overrides the memory_page_max setting.
    // an integer between 512K and 500MB; default 10MB.
    chunk_size: u32,

    // The maximum number of chunks to include in a merge operation.
    // An integer between 2 and 100; default 15.
    merge_max: u16,

    // The minimum number of chunks to include in a merge operation.
    // If set to 0 or 1 half the value of merge_max is used.
    // An integer no more than 100; default 0.
    merge_min: u16,
}
