use libc::{self, c_char, c_short};
use libwiredtiger as wtffi;
use std::ffi::{CStr, CString};
use std::os::raw;
use std::ptr;

macro_rules! unwrap_or_panic {
    ($option:expr, $( $args:expr ),* ) => {
        match $option {
            Some(f) => f($($args),*),
            None => panic!("function pointer is None"),
        }
    };
}

macro_rules! make_result {
    ($err_code:expr, $ok:expr) => {
        if $err_code == 0 {
            Ok($ok)
        } else {
            Err(Error {
                message: error_message($err_code),
            })
        }
    };
}

pub(crate) unsafe fn from_cstr(ptr: *const c_char) -> String {
    let cstr = CStr::from_ptr(ptr as *const _);
    String::from_utf8_lossy(cstr.to_bytes()).into_owned()
}

pub fn error_message(result: i32) -> String {
    unsafe {
        let msg = wtffi::wiredtiger_strerror(result);
        from_cstr(msg)
    }
}

pub struct RawConnection {
    conn: *mut wtffi::WT_CONNECTION,
}

pub struct RawSession {
    session: *mut wtffi::WT_SESSION,
}

pub struct RawCursor {
    cursor: *mut wtffi::WT_CURSOR,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    fn from_code(code: i32) -> Self {
        Self {
            message: error_message(code),
        }
    }
}

struct OpenConfig {
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

pub type Result<T> = std::result::Result<T, Error>;

impl RawConnection {
    pub fn close(&self) -> Result<()> {
        let err_code = unsafe { unwrap_or_panic!((*self.conn).close, self.conn, std::ptr::null()) };
        make_result!(err_code, ())
    }

    pub fn reconfigure(&self, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#a579141678af06217b22869cbc604c6d4
        let config = CString::new(config).unwrap();
        let err_code =
            unsafe { unwrap_or_panic!((*self.conn).reconfigure, self.conn, config.as_ptr()) };
        make_result!(err_code, ())
    }
    pub fn get_home(&self) -> Result<String> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#a488fcba6b5abcdfca439d456564e8640
        let home = unsafe { unwrap_or_panic!((*self.conn).get_home, self.conn) };
        if !home.is_null() {
            let c_str = unsafe { CStr::from_ptr(home) };

            // Convert the `CStr` to a Rust `String`
            match c_str.to_str() {
                Ok(rust_string) => Ok(rust_string.to_owned()),
                Err(e) => Err(Error {
                    message: format!("Failed to convert C string to Rust string: {}", e),
                }),
            }
        } else {
            panic!("received null from calling get_home on WT_CONNECTION");
        }
    }

    pub fn configure_method(
        &self,
        method: &str,
        uri: &str,
        config: &str,
        config_type: &str,
        check: &str,
    ) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#ab81828b0c9dccc1ccf3d8ef863804137

        let method = CString::new(method).unwrap();
        let uri = CString::new(uri).unwrap();
        let config = CString::new(config).unwrap();
        let config_type = CString::new(config_type).unwrap();
        let check = CString::new(check).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.conn).configure_method,
                self.conn,
                method.as_ptr(),
                uri.as_ptr(),
                config.as_ptr(),
                config_type.as_ptr(),
                check.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn is_new(&self) -> bool {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#ae2bacefe9777b8ab32d8b22c292c4f39
        let new_val = unsafe { unwrap_or_panic!((*self.conn).is_new, self.conn) };
        new_val != 0
    }

    // extensions:
    /*
    int 	load_extension (WT_CONNECTION *connection, const char *path, const char *config)
     Load an extension. More...

    int 	add_data_source (WT_CONNECTION *connection, const char *prefix, WT_DATA_SOURCE *data_source, const char *config)
     Add a custom data source. More...

    int 	add_collator (WT_CONNECTION *connection, const char *name, WT_COLLATOR *collator, const char *config)
     Add a custom collation function. More...

    int 	add_compressor (WT_CONNECTION *connection, const char *name, WT_COMPRESSOR *compressor, const char *config)
     Add a compression function. More...

    int 	add_extractor (WT_CONNECTION *connection, const char *name, WT_EXTRACTOR *extractor, const char *config)
     Add a custom extractor for index keys or column groups. More...
     */

    pub fn open(filename: &str, options: &str) -> Result<Self> {
        // outparam destination for wiredtiger_open()
        let mut conn: *mut wtffi::WT_CONNECTION = ptr::null_mut();

        let options = CString::new(options).unwrap();
        let dbpath = CString::new(filename).unwrap();

        // TODO: support a non-null event handler.
        let event_handler: *const wtffi::WT_EVENT_HANDLER = ptr::null();

        let err_code = unsafe {
            wtffi::wiredtiger_open(
                dbpath.as_ptr(),
                event_handler as *mut wtffi::WT_EVENT_HANDLER,
                options.as_ptr(),
                &mut conn,
            )
        };
        make_result!(err_code, RawConnection { conn })
    }

    pub fn open_session(&self) -> Result<RawSession> {
        let mut session: *mut wtffi::WT_SESSION = ptr::null_mut();
        let event_handler: *mut wtffi::WT_EVENT_HANDLER = ptr::null_mut();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.conn).open_session,
                self.conn,
                event_handler,
                ptr::null(),
                &mut session
            )
        };
        make_result!(err_code, RawSession { session })
    }
}

impl RawSession {
    pub fn close(&self) -> Result<()> {
        let err_code =
            unsafe { unwrap_or_panic!((*self.session).close, self.session, std::ptr::null()) };
        make_result!(err_code, ())
    }
    pub fn compact(&self, name: &str, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#aafa7a12a4891a5bfdc98673a5b8f9c69
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).compact,
                self.session,
                name.as_ptr(),
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn drop(&self, name: &str, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#adf785ef53c16d9dcc77e22cc04c87b70
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).drop,
                self.session,
                name.as_ptr(),
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn log_printf(&self, _fmt: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a504625d0b35da78f738d08530a409be9
        todo!()
    }

    pub fn rename(&self, uri: &str, new_uri: &str, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a1d24b02549009f78b7c6463da0247614
        let uri = CString::new(uri).unwrap();
        let new_uri = CString::new(new_uri).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).rename,
                self.session,
                uri.as_ptr(),
                new_uri.as_ptr(),
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn salvage(&self, name: &str, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#ab3399430e474f7005bd5ea20e6ec7a8e
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).salvage,
                self.session,
                name.as_ptr(),
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn truncate(
        &self,
        name: &str,
        start: RawCursor,
        stop: RawCursor,
        config: &str,
    ) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#ac2bad195e24710d52d730fe3a7c1756a
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).truncate,
                self.session,
                name.as_ptr(),
                start.cursor,
                stop.cursor,
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn upgrade(&self, name: &str, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a556046adc68a33bd317865c6a8d9ad69
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).upgrade,
                self.session,
                name.as_ptr(),
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn verify(&self, name: &str, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a0334da4c85fe8af4197c9a7de27467d3
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.session).verify,
                self.session,
                name.as_ptr(),
                config.as_ptr()
            )
        };
        make_result!(err_code, ())
    }

    pub fn begin_transaction(&self, _config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a7e26b16b26b5870498752322fad790bf
        todo!()
    }

    pub fn commit_transaction(&self, _config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a7e26b16b26b5870498752322fad790bf
        todo!()
    }

    pub fn rollback_transaction(&self, _config: &str) -> Result<()> {
        //  https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#ab45f521464ad9e54d9b15efc2ffe20a1
        todo!()
    }

    pub fn checkpoint(&self, _config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a6550c9079198955c5071583941c85bbf
        todo!()
    }

    pub fn transaction_pinned_range(&self, _config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a1d108fab498cfddbb09ee23e3321a88d
        todo!()
    }

    pub fn create(&self, name: &str, config: &str) -> Result<()> {
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        make_result!(
            unsafe {
                unwrap_or_panic!(
                    (*self.session).create,
                    self.session as *mut wtffi::WT_SESSION,
                    name.as_ptr(),
                    config.as_ptr()
                )
            },
            ()
        )
    }

    pub fn open_cursor(&self, uri: &str) -> Result<RawCursor> {
        let uri = CString::new(uri).unwrap();
        let mut cursor: *mut wtffi::WT_CURSOR = ptr::null_mut();
        let cursor_null: *const wtffi::WT_CURSOR = ptr::null();
        let result = unsafe {
            unwrap_or_panic!(
                (*self.session).open_cursor,
                self.session,
                uri.as_ptr(),
                cursor_null as *mut wtffi::WT_CURSOR,
                ptr::null(),
                &mut cursor
            )
        };
        make_result!(result, RawCursor { cursor })
    }
}

pub enum CompareStatus {
    LessThan,
    Equal,
    GreaterThan,
}

impl CompareStatus {
    fn from_code(code: i32) -> Self {
        match code {
            x if x < 0 => Self::LessThan,
            0 => Self::Equal,
            _ => Self::GreaterThan,
        }
    }
}

impl RawCursor {
    pub fn close(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#aeea071f192cab12245a50fbe71c3460b
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).close, self.cursor) };
        make_result!(err_code, ())
    }

    pub fn reconfigure(&self, config: &str) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#ad6a97a309e2c1ada7ca32a422c46612a
        let config = CString::new(config).unwrap();
        let err_code =
            unsafe { unwrap_or_panic!((*self.cursor).reconfigure, self.cursor, config.as_ptr()) };
        make_result!(err_code, ())
    }

    pub fn get_key(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#af19f6f9d9c7fc248ab38879032620b2f
        // todo!();
        let err_code = unsafe {
            let mut some_val: u16 = 0;
            match (*self.cursor).get_key {
                Some(get_key) => get_key(self.cursor, &some_val),
                None => todo!(),
            }
            // (*self.cursor).get_key(self.cursor, &some_val)
        };
        make_result!(err_code, ())
    }

    pub fn get_value(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#af85364a5af50b95bbc46c82e72f75c01
        /*
            Format	C Type	Python type	Notes
            x	N/A	N/A	pad byte, no associated value
            b	int8_t	int	signed byte
            B	uint8_t	int	unsigned byte
            h	int16_t	int	signed 16-bit
            H	uint16_t	int	unsigned 16-bit
            i	int32_t	int	signed 32-bit
            I	uint32_t	int	unsigned 32-bit
            l	int32_t	int	signed 32-bit
            L	uint32_t	int	unsigned 32-bit
            q	int64_t	int	signed 64-bit
            Q	uint64_t	int	unsigned 64-bit
            r	uint64_t	int	record number
            s	char[]	str	fixed-length string
            S	char[]	str	NUL-terminated string
            t	uint8_t	int	fixed-length bit field
            u	WT_ITEM *	bytes	raw byte array
        */

        todo!();
    }

    pub fn compare(&self, other: &RawCursor) -> Result<CompareStatus> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#acd3f345e375e26d223ad5c6f35dc15e8

        let mut comparep: i32 = 0;

        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.cursor).compare,
                self.cursor,
                other.cursor,
                &mut comparep as *mut i32
            )
        };
        make_result!(err_code, CompareStatus::from_code(comparep))
    }

    pub fn equals(&self, other: &RawCursor) -> Result<bool> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a6736da9b394239a201ba97761b7b941b

        let mut equalp: i32 = 0;

        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.cursor).equals,
                self.cursor,
                other.cursor,
                &mut equalp as *mut i32
            )
        };
        make_result!(err_code, equalp == 1)
    }
    pub fn next(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a0503f16bd8f3d05aa3552f229b3a8e1b
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).next, self.cursor) };
        make_result!(err_code, ())
    }
    pub fn prev(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a43d6664d2f68902aa63f933864242e76
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).prev, self.cursor) };
        make_result!(err_code, ())
    }
    pub fn reset(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#afc1b42c22c9c85e1ba08ce3b34437565
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).reset, self.cursor) };
        make_result!(err_code, ())
    }
    pub fn search_near(&self) -> Result<CompareStatus> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a8068ddce20d0775f26f6dac6e5eb209c
        let mut comparep: i32 = 0;

        let err_code = unsafe {
            unwrap_or_panic!(
                (*self.cursor).search_near,
                self.cursor,
                &mut comparep as *mut i32
            )
        };
        make_result!(err_code, CompareStatus::from_code(comparep))
    }

    pub fn insert(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#aac90d9fbcc031570f924db55f8a1cee3
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).insert, self.cursor) };
        make_result!(err_code, ())
    }

    pub fn update(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a444cdc0952e7f8d55d23173516c7037f
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).insert, self.cursor) };
        make_result!(err_code, ())
    }
    pub fn remove(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#abbba24fe607fee519c4c9c4669cd4455
        let err_code = unsafe { unwrap_or_panic!((*self.cursor).remove, self.cursor) };
        make_result!(err_code, ())
    }

    pub fn scan(&self) {
        let key: *mut wtffi::WT_SESSION = ptr::null_mut();
        let val: *mut wtffi::WT_SESSION = ptr::null_mut();
        unsafe {
            unwrap_or_panic!((*self.cursor).reset, self.cursor);
            loop {
                let result = unwrap_or_panic!((*self.cursor).next, self.cursor);
                if result != 0 {
                    break;
                };
                unwrap_or_panic!((*self.cursor).get_key, self.cursor, &key);
                unwrap_or_panic!((*self.cursor).get_key, self.cursor, &val);
            }
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let ckey = CString::new(key).unwrap();
        let cval = CString::new(value).unwrap();

        let err_code = unsafe {
            unwrap_or_panic!((*self.cursor).set_key, self.cursor, ckey.as_ptr());
            unwrap_or_panic!((*self.cursor).set_value, self.cursor, cval.as_ptr());
            unwrap_or_panic!((*self.cursor).insert, self.cursor)
        };
        make_result!(err_code, ())
    }

    pub fn search(&self, key: &str) -> Result<Option<String>> {
        let ckey = CString::new(key).unwrap();
        let mut val: *mut raw::c_char = ptr::null_mut();
        let err_code = unsafe {
            unwrap_or_panic!((*self.cursor).set_key, self.cursor, ckey.as_ptr());
            unwrap_or_panic!((*self.cursor).search, self.cursor)
        };
        if err_code == wtffi::WT_NOTFOUND {
            return Ok(None);
        }
        if err_code != 0 {
            return Err(Error::from_code(err_code));
        }
        unsafe {
            let err_code = unwrap_or_panic!((*self.cursor).get_value, self.cursor, &mut val);
            if err_code != 0 {
                return Err(Error::from_code(err_code));
            }
            let owned_val = CStr::from_ptr(val).to_string_lossy().into_owned();
            Ok(Some(owned_val))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_ok::assert_ok;
    #[test]
    fn test() {
        // Create a temp dir to put the WT files into, open a connection to it.
        let temp_dir = tempfile::tempdir().unwrap();
        let conn = RawConnection::open(temp_dir.path().to_str().unwrap(), "create").unwrap();
        let session = conn.open_session().unwrap();

        // Create a new table string keys and string values
        let create_result = session.create("table:mytable", "key_format=S,value_format=S");
        assert_ok!(create_result);

        let cursor = assert_ok!(session.open_cursor("table:mytable"));
        assert_ok!(cursor.set("tyler", "brock"));
        assert_ok!(cursor.set("mike", "obrien"));
        println!("tyler: {:?}", cursor.search("tyler").unwrap());
        println!("mike: {:?}", cursor.search("mike").unwrap());

        assert_ok!(cursor.close());
        assert_ok!(session.close());
        assert_ok!(conn.close());
    }
}
