use std::ffi::CStr;
use thiserror::Error;

use crate::raw;

#[derive(Debug, Error)]
pub enum Error {
    #[error("WiredTiger error {code}: {msg}")]
    Wt { code: i32, msg: String },

    #[error("not found")]
    NotFound,

    #[error("duplicate key")]
    DuplicateKey,

    #[error("null pointer returned")]
    NullPointer,

    #[error("invalid UTF-8 in path")]
    BadPath,
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn wt_result(rc: std::os::raw::c_int) -> Result<()> {
    match rc {
        0 => Ok(()),
        raw::WT_NOTFOUND => Err(Error::NotFound),
        raw::WT_DUPLICATE_KEY => Err(Error::DuplicateKey),
        code => {
            let msg = unsafe {
                let ptr = raw::wiredtiger_strerror(code);
                if ptr.is_null() {
                    format!("unknown error {code}")
                } else {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            };
            Err(Error::Wt { code, msg })
        }
    }
}
