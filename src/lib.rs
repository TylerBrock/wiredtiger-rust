use libc::{self, c_char};
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

pub struct Connection {
    conn: *mut wtffi::WT_CONNECTION,
}

pub struct Session {
    session: *mut wtffi::WT_SESSION,
}

pub struct Cursor {
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

type Result<T> = std::result::Result<T, Error>;

impl Connection {
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
        make_result!(err_code, Connection { conn })
    }

    pub fn open_session(&self) -> Result<Session> {
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
        make_result!(err_code, Session { session })
    }
}

impl Session {
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

    pub fn truncate(&self, name: &str, start: Cursor, stop: Cursor, config: &str) -> Result<()> {
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

    pub fn open_cursor(&self, uri: &str) -> Result<Cursor> {
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
        make_result!(result, Cursor { cursor })
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

impl Cursor {
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
        todo!();
    }

    pub fn get_value(&self) -> Result<()> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#af85364a5af50b95bbc46c82e72f75c01
        todo!();
    }

    pub fn compare(&self, other: Cursor) -> Result<CompareStatus> {
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

    pub fn equals(&self, other: Cursor) -> Result<bool> {
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

// impl Drop for Connection {
//     fn drop(&mut self) {
//         unsafe {
//             unwrap_or_panic!((*self.conn).close, self.conn, std::ptr::null());
//         }
//     }
// }

// impl Drop for Session {
//     fn drop(&mut self) {
//         unsafe {
//             unwrap_or_panic!((*self.session).close, self.session, std::ptr::null());
//         }
//     }
// }

// impl Drop for Cursor {
//     fn drop(&mut self) {
//         unsafe {
//             unwrap_or_panic!((*self.cursor).close, self.cursor);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::*;
    use assert_ok::assert_ok;
    #[test]
    fn test() {
        // Create a temp dir to put the WT files into, open a connection to it.
        let temp_dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(temp_dir.path().to_str().unwrap(), "create").unwrap();
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
