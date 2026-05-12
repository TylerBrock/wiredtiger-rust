use std::ffi::CString;
use std::ptr;

use crate::error::{wt_result, Error, Result};
use crate::raw;
use crate::session::Session;

/// A WiredTiger connection. Thread-safe; share via `Arc`.
pub struct Connection {
    pub(crate) ptr: *mut raw::WT_CONNECTION,
}

// WT_CONNECTION is explicitly documented as thread-safe.
unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}

impl Connection {
    /// Open (or create) a WiredTiger database at `home`.
    pub fn open(home: &str, config: &str) -> Result<Self> {
        let home_c = CString::new(home).map_err(|_| Error::BadPath)?;
        let cfg_c = CString::new(config).map_err(|_| Error::BadPath)?;
        let mut conn: *mut raw::WT_CONNECTION = ptr::null_mut();
        let rc = unsafe {
            raw::wiredtiger_open(
                home_c.as_ptr(),
                ptr::null_mut(),
                cfg_c.as_ptr(),
                &mut conn,
            )
        };
        wt_result(rc)?;
        if conn.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Connection { ptr: conn })
    }

    /// Open a new session on this connection.
    pub fn open_session(&self) -> Result<Session> {
        let mut sess: *mut raw::WT_SESSION = ptr::null_mut();
        let rc = unsafe {
            let open_session = (*self.ptr).open_session.expect("open_session fn ptr null");
            open_session(self.ptr, ptr::null_mut(), ptr::null(), &mut sess)
        };
        wt_result(rc)?;
        if sess.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Session { ptr: sess })
    }

    /// Close the connection (called automatically on drop).
    pub fn close(self) -> Result<()> {
        let rc = unsafe {
            let close = (*self.ptr).close.expect("close fn ptr null");
            close(self.ptr, ptr::null())
        };
        // Prevent the Drop impl from double-closing
        std::mem::forget(self);
        wt_result(rc)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                if let Some(close) = (*self.ptr).close {
                    close(self.ptr, ptr::null());
                }
            }
        }
    }
}
