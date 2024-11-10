use libc::{self, c_char, c_void, size_t};
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
    if result != 0 {
        return Err(get_error(result));
    }
    return Ok(value);
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
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl Connection {
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
            if let Some(open_session) = (*self.conn).open_session {
                let result = open_session(self.conn, event_handler, ptr::null(), &mut session);
                return make_result(result, Session { session });
            } else {
                Err(Error::new("open_session not found".to_string()))
            }
        }
    }
}

impl Session {
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
            return make_result(result, Some(owned_val));
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
