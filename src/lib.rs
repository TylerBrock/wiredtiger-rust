mod wiredtiger;

use wiredtiger::{
    WT_CONNECTION,
    WT_SESSION,
    WT_CURSOR,
    WT_EVENT_HANDLER,
};
use wiredtiger::{
    wiredtiger_open,
    wiredtiger_strerror,
};
use wiredtiger::WT_NOTFOUND;

use std::ffi::{ CString, CStr };
use std::ptr;
use std::io;
use std::os::raw;

fn get_error(result: i32) -> io::Error {
    let err_msg = unsafe { CStr::from_ptr(wiredtiger_strerror(result)) };
    return io::Error::other(err_msg.to_str().unwrap().to_owned());
}

fn make_result<T>(result: i32, value: T) -> Result<T, io::Error> {
    if result != 0 {
        return Err(get_error(result));
    }
    return Ok(value);
}

pub struct Connection {
    conn: *mut WT_CONNECTION,
}

pub struct Session {
    session: *mut WT_SESSION,
}

pub struct Cursor {
    cursor: *mut WT_CURSOR,
}

impl Connection {
    pub fn open(filename: &str, options: &str) -> Result<Self, io::Error> {
        let mut conn: *mut WT_CONNECTION = ptr::null_mut();
        let options = CString::new(options).unwrap();
        let dbpath = CString::new(filename).unwrap();
        let event_handler: *const WT_EVENT_HANDLER = ptr::null();
        let result: i32;
        unsafe {
            result = wiredtiger_open(
                dbpath.as_ptr(),
                event_handler as *mut WT_EVENT_HANDLER,
                options.as_ptr(),
                &mut conn,
            );
        };
        make_result(result, Connection { conn })
    }

    pub fn create_session(&self) -> Result<Session, io::Error> {
        let mut session: *mut WT_SESSION = ptr::null_mut();
        let event_handler: *mut WT_EVENT_HANDLER = ptr::null_mut();
        let result: i32;
        unsafe {
            result = (self.conn.as_ref().unwrap().open_session.unwrap())(
                self.conn,
                event_handler,
                ptr::null(),
                &mut session,
            );
        }
        make_result(result, Session { session })
    }
}

impl Session {
    pub fn create(&self, name: &str, config: &str) -> Result<(), io::Error> {
        let name = CString::new(name).unwrap();
        let config = CString::new(config).unwrap();
        let result: i32;
        unsafe {
            result = (self.session.as_ref().unwrap().create.unwrap())(
                self.session as *mut WT_SESSION,
                name.as_ptr(),
                config.as_ptr(),
            );
        }
        make_result(result, ())
    }

    pub fn cursor(&self, uri: &str) -> Result<Cursor, io::Error> {
        let uri = CString::new(uri).unwrap();
        let mut cursor: *mut WT_CURSOR = ptr::null_mut();
        let cursor_null: *const WT_CURSOR = ptr::null();
        let result: i32;
        unsafe {
            result = (self.session.as_ref().unwrap().open_cursor.unwrap())(
                self.session,
                uri.as_ptr(),
                cursor_null as *mut WT_CURSOR,
                ptr::null(),
                &mut cursor,
            );
        }
        make_result(result, Cursor{cursor})
    }
}

impl Cursor {
    pub fn scan(&self) {
        let key: *mut WT_SESSION = ptr::null_mut();
        let val: *mut WT_SESSION = ptr::null_mut();
        unsafe {
            self.cursor.as_ref().unwrap().reset.unwrap()(
                self.cursor as *mut WT_CURSOR
            );
            loop {
                let result = self.cursor.as_ref().unwrap().next.unwrap()(
                    self.cursor as *mut WT_CURSOR
                );
                if result != 0 { break };
                self.cursor.as_ref().unwrap().get_key.unwrap()(
                    self.cursor as *mut WT_CURSOR,
                    &key
                );
                self.cursor.as_ref().unwrap().get_key.unwrap()(
                    self.cursor as *mut WT_CURSOR,
                    &val
                );
            }
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), io::Error> {
        let result: i32;
        let ckey = CString::new(key).unwrap();
        let cval = CString::new(value).unwrap();
        unsafe {
            self.cursor.as_ref().unwrap().set_key.unwrap()(
                self.cursor as *mut WT_CURSOR,
                ckey.as_ptr(),
            );
            self.cursor.as_ref().unwrap().set_value.unwrap()(
                self.cursor as *mut WT_CURSOR,
                cval.as_ptr(),
            );
            result = self.cursor.as_ref().unwrap().insert.unwrap()(
                self.cursor as *mut WT_CURSOR
            );
        }
        make_result(result, ())
    }

    pub fn search(&self, key: &str) -> Result<Option<String>, io::Error> {
        let mut result: i32;
        let ckey = CString::new(key).unwrap();
        let mut val: *mut raw::c_char = ptr::null_mut();
        unsafe {
            self.cursor.as_ref().unwrap().set_key.unwrap()(
                self.cursor as *mut WT_CURSOR,
                ckey.as_ptr(),
            );
            result = self.cursor.as_ref().unwrap().search.unwrap()(
                self.cursor as *mut WT_CURSOR
            );
            if result == WT_NOTFOUND {
                return Ok(None);
            }
            if result !=0 {
                return Err(get_error(result));
            }
            result = self.cursor.as_ref().unwrap().get_value.unwrap()(
                self.cursor as *mut WT_CURSOR,
                &mut val,
            );
            let owned_val = CStr::from_ptr(val).to_string_lossy().into_owned();
            make_result(result, Some(owned_val))
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        println!("closing connection!");
        unsafe {
            // *WT_CONNECTION.as_ref() -> Option<&WT_CONNECTION>
            // conn->close(conn, null);
            (self.conn.as_ref().unwrap().close.unwrap())(self.conn, std::ptr::null());
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        //todo!()
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        // WT_CONNECTION::close
        // (self.conn as WT_CONNECTION)
        //todo!()
    }
}
