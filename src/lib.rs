use libc::{self, c_char};
use libwiredtiger as wtffi;
use std::ffi::{CStr, CString};
use std::os::raw;
use std::ptr;

mod raw_api;

use raw_api::{RawConnection, Result};

struct Connection {
    raw_conn: raw_api::RawConnection,
}

impl Connection {
    pub fn open(filename: &str, options: &str) -> Result<Self> {
        let raw_conn = RawConnection::open(filename, options)?;
        Ok(Self { raw_conn })
    }
}

struct Cursor {
    raw_cursor: raw_api::RawCursor,
}

struct Session {
    raw_session: raw_api::RawSession,
}
