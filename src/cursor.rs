use std::ptr;
use std::slice;

use crate::error::{wt_result, Error, Result};
use crate::raw;

/// A WiredTiger cursor. Stays open between operations for efficiency.
pub struct Cursor {
    pub(crate) ptr: *mut raw::WT_CURSOR,
}

unsafe impl Send for Cursor {}

impl Cursor {
    /// Set the key for the next operation (raw bytes).
    pub fn set_key(&mut self, key: &[u8]) {
        let item = raw::WT_ITEM {
            data: key.as_ptr() as *const _,
            size: key.len(),
            mem: ptr::null_mut(),
            memsize: 0,
            flags: 0,
        };
        unsafe {
            let set_key = (*self.ptr).set_key.expect("set_key fn ptr null");
            set_key(self.ptr, &item as *const raw::WT_ITEM);
        }
    }

    /// Set the value for the next insert/update (raw bytes).
    pub fn set_value(&mut self, value: &[u8]) {
        let item = raw::WT_ITEM {
            data: value.as_ptr() as *const _,
            size: value.len(),
            mem: ptr::null_mut(),
            memsize: 0,
            flags: 0,
        };
        unsafe {
            let set_value = (*self.ptr).set_value.expect("set_value fn ptr null");
            set_value(self.ptr, &item as *const raw::WT_ITEM);
        }
    }

    /// Get the value at the current cursor position. The slice is valid until the next cursor op.
    pub fn get_value(&mut self) -> Result<&[u8]> {
        let mut item = raw::WT_ITEM {
            data: ptr::null(),
            size: 0,
            mem: ptr::null_mut(),
            memsize: 0,
            flags: 0,
        };
        let rc = unsafe {
            let get_value = (*self.ptr).get_value.expect("get_value fn ptr null");
            get_value(self.ptr, &mut item as *mut raw::WT_ITEM)
        };
        wt_result(rc)?;
        if item.data.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(unsafe { slice::from_raw_parts(item.data as *const u8, item.size) })
    }

    /// Insert or overwrite the current key/value. Returns Ok(()) on success.
    pub fn insert(&mut self) -> Result<()> {
        let rc = unsafe {
            let insert = (*self.ptr).insert.expect("insert fn ptr null");
            insert(self.ptr)
        };
        wt_result(rc)
    }

    /// Search for the exact key set via `set_key`. Returns `Ok(true)` if found.
    pub fn search(&mut self) -> Result<bool> {
        let rc = unsafe {
            let search = (*self.ptr).search.expect("search fn ptr null");
            search(self.ptr)
        };
        match wt_result(rc) {
            Ok(()) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Remove the record at the current key. Returns `Ok(true)` if a record was deleted.
    pub fn remove(&mut self) -> Result<bool> {
        let rc = unsafe {
            let remove = (*self.ptr).remove.expect("remove fn ptr null");
            remove(self.ptr)
        };
        match wt_result(rc) {
            Ok(()) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Advance to the next record. Returns `Ok(true)` if a record was found.
    pub fn next(&mut self) -> Result<bool> {
        let rc = unsafe {
            let next = (*self.ptr).next.expect("next fn ptr null");
            next(self.ptr)
        };
        match wt_result(rc) {
            Ok(()) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Reset the cursor position; releases any held resources.
    pub fn reset(&mut self) -> Result<()> {
        let rc = unsafe {
            let reset = (*self.ptr).reset.expect("reset fn ptr null");
            reset(self.ptr)
        };
        wt_result(rc)
    }

    /// Position at the key (or nearest). Returns `Some(ordering)` where ordering < 0 means
    /// cursor landed before the key, 0 = exact, > 0 = after. Returns `None` if table is empty.
    pub fn search_near(&mut self) -> Result<Option<i32>> {
        let mut exact: std::os::raw::c_int = 0;
        let rc = unsafe {
            let search_near = (*self.ptr).search_near.expect("search_near fn ptr null");
            search_near(self.ptr, &mut exact)
        };
        match wt_result(rc) {
            Ok(()) => Ok(Some(exact)),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get both key and value at the current position as owned byte vectors.
    pub fn get_raw_kv(&mut self) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut key_item = raw::WT_ITEM {
            data: ptr::null(),
            size: 0,
            mem: ptr::null_mut(),
            memsize: 0,
            flags: 0,
        };
        let mut val_item = raw::WT_ITEM {
            data: ptr::null(),
            size: 0,
            mem: ptr::null_mut(),
            memsize: 0,
            flags: 0,
        };
        let rc = unsafe {
            let grk = (*self.ptr).get_raw_key_value.expect("get_raw_key_value fn ptr null");
            grk(self.ptr, &mut key_item, &mut val_item)
        };
        wt_result(rc)?;
        let k = unsafe { slice::from_raw_parts(key_item.data as *const u8, key_item.size).to_vec() };
        let v = unsafe { slice::from_raw_parts(val_item.data as *const u8, val_item.size).to_vec() };
        Ok((k, v))
    }

    pub fn close(self) -> Result<()> {
        let rc = unsafe {
            let close = (*self.ptr).close.expect("close fn ptr null");
            close(self.ptr)
        };
        std::mem::forget(self);
        wt_result(rc)
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                if let Some(close) = (*self.ptr).close {
                    close(self.ptr);
                }
            }
        }
    }
}
