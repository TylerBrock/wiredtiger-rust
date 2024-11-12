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

fn get_error(result: i32) -> Error {
    Error {
        message: error_message(result),
    }
}

// TODO make this a macro?
fn make_result<T>(result: i32, value: T) -> Result<T, Error> {
    if result == 0 {
        Ok(value)
    } else {
        Err(get_error(result))
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

impl Connection {
    pub fn reconfigure(_options: &str) -> Result<Self, Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#a579141678af06217b22869cbc604c6d4
        todo!()
    }
    pub fn get_home() -> Result<Self, Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#a488fcba6b5abcdfca439d456564e8640
        todo!()
    }
    pub fn configure_method() -> Result<Self, Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#ab81828b0c9dccc1ccf3d8ef863804137
        todo!()
    }

    pub fn is_new() -> Result<Self, Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_o_n_n_e_c_t_i_o_n.html#ae2bacefe9777b8ab32d8b22c292c4f39
        todo!()
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

    pub fn open(filename: &str, options: &str) -> Result<Self, Error> {
        // outparam destination for wiredtiger_open()
        let mut conn: *mut wtffi::WT_CONNECTION = ptr::null_mut();

        let options = CString::new(options).unwrap();
        let dbpath = CString::new(filename).unwrap();

        // TODO: support a non-null event handler.
        let event_handler: *const wtffi::WT_EVENT_HANDLER = ptr::null();

        let result = unsafe {
            wtffi::wiredtiger_open(
                dbpath.as_ptr(),
                event_handler as *mut wtffi::WT_EVENT_HANDLER,
                options.as_ptr(),
                &mut conn,
            )
        };
        make_result(result, Connection { conn })
    }

    pub fn open_session(&self) -> Result<Session, Error> {
        let mut session: *mut wtffi::WT_SESSION = ptr::null_mut();
        let event_handler: *mut wtffi::WT_EVENT_HANDLER = ptr::null_mut();
        unsafe {
            let result = unwrap_or_panic!(
                (*self.conn).open_session,
                self.conn,
                event_handler,
                ptr::null(),
                &mut session
            );
            make_result(result, Session { session })
        }
    }
}

impl Session {
    pub fn compact(&self, name: &str, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#aafa7a12a4891a5bfdc98673a5b8f9c69
        todo!()
    }

    pub fn drop(&self, name: &str, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#adf785ef53c16d9dcc77e22cc04c87b70
        todo!()
    }

    pub fn log_printf(&self, fmt: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a504625d0b35da78f738d08530a409be9
        todo!()
    }

    pub fn rename(&self, uri: &str, newuri: &str, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a1d24b02549009f78b7c6463da0247614
        todo!()
    }

    pub fn salvage(&self, name: &str, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#ab3399430e474f7005bd5ea20e6ec7a8e
        todo!()
    }

    pub fn truncate(&self, name: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#ac2bad195e24710d52d730fe3a7c1756a
        todo!()
    }

    pub fn upgrade(&self, name: &str, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a556046adc68a33bd317865c6a8d9ad69
        todo!()
    }

    pub fn verify(&self, name: &str, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a0334da4c85fe8af4197c9a7de27467d3
        todo!()
    }

    pub fn begin_transaction(&self, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a7e26b16b26b5870498752322fad790bf
        todo!()
    }

    pub fn commit_transaction(&self, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a7e26b16b26b5870498752322fad790bf
        todo!()
    }

    pub fn rollback_transaction(&self, config: &str) -> Result<(), Error> {
        //  https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#ab45f521464ad9e54d9b15efc2ffe20a1
        todo!()
    }

    pub fn checkpoint(&self, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a6550c9079198955c5071583941c85bbf
        todo!()
    }

    pub fn transaction_pinned_range(&self, config: &str) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___s_e_s_s_i_o_n.html#a1d108fab498cfddbb09ee23e3321a88d
        todo!()
    }

    pub fn create(&self, name: &str, config: &str) -> Result<(), Error> {
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let result = unsafe {
            unwrap_or_panic!(
                (*self.session).create,
                self.session as *mut wtffi::WT_SESSION,
                name.as_ptr(),
                config.as_ptr()
            )
        };
        make_result(result, ())
    }

    pub fn open_cursor(&self, uri: &str) -> Result<Cursor, Error> {
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
        make_result(result, Cursor { cursor })
    }
}

impl Cursor {
    pub fn close(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#aeea071f192cab12245a50fbe71c3460b
        todo!();
    }
    pub fn reconfigure(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#ad6a97a309e2c1ada7ca32a422c46612a
        todo!();
    }
    pub fn get_key(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#af19f6f9d9c7fc248ab38879032620b2f
        todo!();
    }
    pub fn get_value(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#af85364a5af50b95bbc46c82e72f75c01
        todo!();
    }
    pub fn compare(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#acd3f345e375e26d223ad5c6f35dc15e8
        todo!();
    }
    pub fn equals(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a6736da9b394239a201ba97761b7b941b
        todo!();
    }
    pub fn next(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a0503f16bd8f3d05aa3552f229b3a8e1b
        todo!();
    }
    pub fn prev(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a43d6664d2f68902aa63f933864242e76
        todo!();
    }
    pub fn reset(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#afc1b42c22c9c85e1ba08ce3b34437565
        todo!();
    }
    pub fn search_near(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a8068ddce20d0775f26f6dac6e5eb209c
        todo!();
    }
    pub fn insert(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#aac90d9fbcc031570f924db55f8a1cee3
        todo!();
    }

    pub fn update(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#a444cdc0952e7f8d55d23173516c7037f
        todo!();
    }
    pub fn remove(&self) -> Result<(), Error> {
        // https://source.wiredtiger.com/2.5.2/struct_w_t___c_u_r_s_o_r.html#abbba24fe607fee519c4c9c4669cd4455
        todo!();
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

    pub fn set(&self, key: &str, value: &str) -> Result<(), Error> {
        let ckey = CString::new(key).unwrap();
        let cval = CString::new(value).unwrap();

        let result = unsafe {
            unwrap_or_panic!((*self.cursor).set_key, self.cursor, ckey.as_ptr());
            unwrap_or_panic!((*self.cursor).set_value, self.cursor, cval.as_ptr());
            unwrap_or_panic!((*self.cursor).insert, self.cursor)
        };
        make_result(result, ())
    }

    pub fn search(&self, key: &str) -> Result<Option<String>, Error> {
        let ckey = CString::new(key).unwrap();
        let mut val: *mut raw::c_char = ptr::null_mut();
        let result = unsafe {
            unwrap_or_panic!((*self.cursor).set_key, self.cursor, ckey.as_ptr());
            unwrap_or_panic!((*self.cursor).search, self.cursor)
        };
        if result == wtffi::WT_NOTFOUND {
            return Ok(None);
        }
        if result != 0 {
            return Err(get_error(result));
        }
        unsafe {
            let result = unwrap_or_panic!((*self.cursor).get_value, self.cursor, &mut val);
            if result != 0 {
                return Err(get_error(result));
            }
            let owned_val = CStr::from_ptr(val).to_string_lossy().into_owned();
            make_result(result, Some(owned_val))
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            unwrap_or_panic!((*self.conn).close, self.conn, std::ptr::null());
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            unwrap_or_panic!((*self.session).close, self.session, std::ptr::null());
        }
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe {
            unwrap_or_panic!((*self.cursor).close, self.cursor);
        }
    }
}

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
    }
}
