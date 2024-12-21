mod raw_api;

mod config;

use delegate::delegate;
pub use raw_api::Error;
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

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "connection")
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

    pub fn create(&self, name: &str, config: &str) -> Result<()> {
        self.raw_session.create(name, config)
    }
}

impl<'a> Cursor<'a> {
    pub fn compare(&self, other: Cursor) -> Result<CompareStatus> {
        self.raw_cursor.compare(&other.raw_cursor)
    }

    pub fn equals(&self, other: Cursor) -> Result<bool> {
        self.raw_cursor.equals(&other.raw_cursor)
    }

    delegate! {
        to self.raw_cursor{
            pub fn bound(&self, config: &str);
            pub fn insert(&self) -> Result<()>;
            pub fn largest_key(&self) -> Result<()>;
            // int WT_CURSOR::modify	(	WT_CURSOR * 	cursor, WT_MODIFY * 	entries, int 	nentries )
            pub fn next(&self) -> Result<()>;
            pub fn prev(&self) -> Result<()>;
    // int WT_CURSOR::reconfigure	(	WT_CURSOR * 	cursor, const char * 	config )
    // int WT_CURSOR::remove	(	WT_CURSOR * 	cursor	)
    // int WT_CURSOR::reserve	(	WT_CURSOR * 	cursor	)
            pub fn reset(&self) -> Result<()> ;
    // int WT_CURSOR::search	(	WT_CURSOR * 	cursor	)
    // int WT_CURSOR::search_near	(	WT_CURSOR * 	cursor, int * 	exactp )
    // int WT_CURSOR::update	(	WT_CURSOR * 	cursor	)
            pub fn set_key(&self, key: &str);
            pub fn set_value(&self, key: &str);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{Connection, Error};
    use assert_ok::assert_ok;
    use tempfile::TempDir;

    // Tests that opening a database (without "create")
    // returns an error when the file does not exist.
    #[test]
    fn test_open_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let res = Connection::open(temp_dir.path().to_str().unwrap().into(), "");
        if let Err(Error { message }) = res {
            assert_eq!(message, "WT_TRY_SALVAGE: database corruption detected");
        } else {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_basic() {
        // Create a temp dir to put the WT files into, open a connection to it.
        let temp_dir = tempfile::tempdir().unwrap();

        let conn = Connection::open(temp_dir.path().to_str().unwrap().into(), "create")
            .expect("failed to open connection");
        let sess = assert_ok!(conn.open_session());
        assert_ok!(sess.create("table:foo", ""));

        let create_result = sess.create("table:mytable", "key_format=S,value_format=S");
        assert_ok!(create_result);
        let cur = assert_ok!(sess.open_cursor("table:mytable"));

        //cur.set_key("tyler");
        //cur.set_value("brock");
        //assert_ok!(cur.insert());
        //assert_ok!(cur.set("mike", "obrien"));
        //println!("tyler: {:?}", cursor.search("tyler").unwrap());
        //println!("mike: {:?}", cursor.search("mike").unwrap());
        //
        //assert_ok!(cursor.close());
        //assert_ok!(session.close());
        //assert_ok!(conn.close());
    }
}

//use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, ItemFn};

//#[proc_macro_attribute]
//pub fn wiredtiger_format(
//    attr: proc_macro::TokenStream,
//    item: proc_macro::TokenStream,
//) -> proc_macro::TokenStream {
//    println!("attr: \"{attr}\"");
//    println!("item: \"{item}\"");
//
//    item
//}
//
