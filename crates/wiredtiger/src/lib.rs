mod raw_api;

mod config;

use raw_api::{CompareStatus, RawConnection, Result};

struct Connection {
    raw_conn: raw_api::RawConnection,
}

impl Connection {
    pub fn open(filename: &str, options: &str) -> Result<Self> {
        let raw_conn = RawConnection::open(filename, options)?;
        Ok(Self { raw_conn })
    }
    pub fn open_session(&self) -> Result<Session> {
        let raw_session = self.raw_conn.open_session()?;
        Ok(Session {
            raw_session,
            conn: &self,
        })
    }
}

impl<'a> Session<'a> {
    pub fn open_cursor(&self, uri: &str) -> Result<Cursor> {
        let raw_cursor = self.raw_session.open_cursor(uri)?;
        Ok(Cursor {
            session: &self,
            raw_cursor,
        })
    }
}

impl<'a> Cursor<'a> {
    pub fn compare(&self, other: Cursor) -> Result<CompareStatus> {
        self.raw_cursor.compare(&other.raw_cursor)
    }

    pub fn equals(&self, other: Cursor) -> Result<bool> {
        self.raw_cursor.equals(&other.raw_cursor)
    }

    pub fn next(&self) -> Result<()> {
        self.raw_cursor.next()
    }

    pub fn prev(&self) -> Result<()> {
        self.raw_cursor.prev()
    }

    pub fn reset(&self) -> Result<()> {
        self.raw_cursor.reset()
    }

    // pub fn search_near(&self) -> Result<CompareStatus> {
    //     self.search_near
    // pub fn insert(&self) -> Result<()> {
    // pub fn update(&self) -> Result<()> {
    // pub fn remove(&self) -> Result<()> {
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.raw_conn.close().unwrap();
    }
}

impl<'a> Drop for Session<'a> {
    fn drop(&mut self) {
        self.raw_session.close().unwrap();
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        self.raw_cursor.close().unwrap();
    }
}

struct Cursor<'a> {
    session: &'a Session<'a>,
    raw_cursor: raw_api::RawCursor,
}

struct Session<'a> {
    raw_session: raw_api::RawSession,
    conn: &'a Connection,
}

use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn wiredtiger_format(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    println!("attr: \"{attr}\"");
    println!("item: \"{item}\"");

    item
}
