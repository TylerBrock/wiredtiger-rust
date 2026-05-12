use std::ffi::CString;
use std::ptr;

use crate::cursor::Cursor;
use crate::error::{wt_result, Error, Result};
use crate::raw;

/// A WiredTiger session. NOT thread-safe — use one per thread or protect with a Mutex.
pub struct Session {
    pub(crate) ptr: *mut raw::WT_SESSION,
}

// We wrap Session in Mutex in the backend, so Send is fine.
unsafe impl Send for Session {}

impl Session {
    /// Create a table/index. Config example: `"key_format=u,value_format=u"`.
    pub fn create(&self, uri: &str, config: &str) -> Result<()> {
        let uri_c = CString::new(uri).map_err(|_| Error::BadPath)?;
        let cfg_c = CString::new(config).map_err(|_| Error::BadPath)?;
        let rc = unsafe {
            let create = (*self.ptr).create.expect("create fn ptr null");
            create(self.ptr, uri_c.as_ptr(), cfg_c.as_ptr())
        };
        wt_result(rc)
    }

    /// Drop a table. Returns Ok(()) even if the table doesn't exist (with `force=true`).
    pub fn drop_table(&self, uri: &str) -> Result<()> {
        let uri_c = CString::new(uri).map_err(|_| Error::BadPath)?;
        let cfg_c = CString::new("force=true").map_err(|_| Error::BadPath)?;
        let rc = unsafe {
            let drop = (*self.ptr).drop.expect("drop fn ptr null");
            drop(self.ptr, uri_c.as_ptr(), cfg_c.as_ptr())
        };
        wt_result(rc)
    }

    /// Open a cursor on `uri`. Config: `""` for default, `"overwrite=false"` for no-upsert.
    pub fn open_cursor(&self, uri: &str, config: &str) -> Result<Cursor> {
        let uri_c = CString::new(uri).map_err(|_| Error::BadPath)?;
        let cfg_c = CString::new(config).map_err(|_| Error::BadPath)?;
        let mut cursor: *mut raw::WT_CURSOR = ptr::null_mut();
        let rc = unsafe {
            let open = (*self.ptr).open_cursor.expect("open_cursor fn ptr null");
            open(self.ptr, uri_c.as_ptr(), ptr::null_mut(), cfg_c.as_ptr(), &mut cursor)
        };
        wt_result(rc)?;
        if cursor.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Cursor { ptr: cursor })
    }

    pub fn begin_transaction(&self) -> Result<()> {
        let rc = unsafe {
            let begin = (*self.ptr).begin_transaction.expect("begin_transaction fn ptr null");
            begin(self.ptr, ptr::null())
        };
        wt_result(rc)
    }

    pub fn commit_transaction(&self) -> Result<()> {
        let rc = unsafe {
            let commit = (*self.ptr).commit_transaction.expect("commit_transaction fn ptr null");
            commit(self.ptr, ptr::null())
        };
        wt_result(rc)
    }

    pub fn rollback_transaction(&self) -> Result<()> {
        let rc = unsafe {
            let rollback = (*self.ptr).rollback_transaction.expect("rollback_transaction fn ptr null");
            rollback(self.ptr, ptr::null())
        };
        wt_result(rc)
    }

    pub fn close(self) -> Result<()> {
        let rc = unsafe {
            let close = (*self.ptr).close.expect("close fn ptr null");
            close(self.ptr, ptr::null())
        };
        std::mem::forget(self);
        wt_result(rc)
    }
}

impl Drop for Session {
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
