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

    delegate! {
        to self.raw_conn {
            pub fn get_home(&self) -> Result<String>;
            pub fn is_new(&self) -> bool ;
            pub fn reconfigure(&self, config: &str) -> Result<()>;
        }
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

    delegate! {
        to self.raw_session{
            pub fn create(&self, name: &str, config: &str) -> Result<()>;
            pub fn compact(&self, name: &str, config: &str) -> Result<()>;
            pub fn drop(&self, name: &str, config: &str) -> Result<()>;
            pub fn reconfigure(&self,  config: &str) -> Result<()>;
            pub fn reset(&self) -> Result<()>;
            pub fn reset_snapshot(&self) -> Result<()>;
        }
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
            pub fn bound(&self, config: &str) -> Result<()> ;
            pub fn get_raw_key_value(&self) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>)>;
            pub fn insert(&self) -> Result<()>;
            pub fn largest_key(&self) -> Result<()>;
            // int WT_CURSOR::modify	(	WT_CURSOR * 	cursor, WT_MODIFY * 	entries, int 	nentries )
            pub fn next(&self) -> Result<()>;
            pub fn prev(&self) -> Result<()>;
            pub fn reconfigure(&self, config: &str) -> Result<()>;
            pub fn remove(&self) -> Result<()>;
            pub fn reserve(&self) -> Result<()>;
            pub fn reset(&self) -> Result<()> ;
            pub fn search(&self) -> Result<()> ;
            pub fn search_near(&self) -> Result<CompareStatus> ;
            pub fn update(&self) -> Result<()>;
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

    // Tests that opening a database (without "create")
    // returns an error when the file does not exist.
    #[test]
    fn test_open_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let res = Connection::open(temp_dir.path().to_str().unwrap().into(), "");
        if let Err(Error { code, message }) = res {
            assert_eq!(message, "WT_TRY_SALVAGE: database corruption detected");
        } else {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_basic() {
        // Create a temp dir to put the WT files into, open a connection to it.
        let temp_dir = tempfile::tempdir().unwrap();

        {
            let conn = Connection::open(temp_dir.path().to_str().unwrap().into(), "create")
                .expect("failed to open connection");
            let sess = assert_ok!(conn.open_session());
            assert_ok!(sess.create("table:foo", ""));

            let create_result = sess.create("table:mytable", "key_format=S,value_format=S");
            assert_ok!(create_result);
            let cur = assert_ok!(sess.open_cursor("table:mytable"));

            cur.set_key("tyler");
            cur.set_value("brock");
            assert_ok!(cur.insert());

            cur.set_key("mike");
            cur.set_value("obrien");
            assert_ok!(cur.insert());

            cur.set_key("tyler");
            assert_ok!(cur.search());

            let (k, v) = assert_ok!(cur.get_raw_key_value());
            let (k, v) = (k.unwrap(), v.unwrap());

            assert_eq!(assert_ok!(std::str::from_utf8(&k)), "tyler");
            assert_eq!(assert_ok!(std::str::from_utf8(&v)), "brock");
        }

        // Re-open the file and assert the data is still in there
        {
            let conn = Connection::open(temp_dir.path().to_str().unwrap().into(), "create")
                .expect("failed to open connection");
            let sess = assert_ok!(conn.open_session());
            let cur = assert_ok!(sess.open_cursor("table:mytable"));

            assert_ok!(cur.next());
            let (k, v) = assert_ok!(cur.get_raw_key_value());
            let (k, v) = (k.unwrap(), v.unwrap());
            assert_eq!(assert_ok!(std::str::from_utf8(&k)), "mike");
            assert_eq!(assert_ok!(std::str::from_utf8(&v)), "obrien");

            assert_ok!(cur.next());
            let (k, v) = assert_ok!(cur.get_raw_key_value());
            let (k, v) = (k.unwrap(), v.unwrap());
            assert_eq!(assert_ok!(std::str::from_utf8(&k)), "tyler");
            assert_eq!(assert_ok!(std::str::from_utf8(&v)), "brock");
        }
    }

    #[test]
    fn test_reconfigure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(temp_dir.path().to_str().unwrap().into(), "create")
            .expect("failed to open connection");
        let sess = assert_ok!(conn.open_session());

        // Calling connection reconfigure with an invalid config string fails
        assert!(matches!(
            conn.reconfigure("bogus"),
            Err(Error {
                code,
                message,
            })
            if message == "Invalid argument" && code == libc::EINVAL
        ));

        // Calling session reconfigure with an invalid config string fails
        assert!(matches!(
            sess.reconfigure("bogus"),
            Err(Error {
                code,
                message,
            })
            if message == "Invalid argument" && code == libc::EINVAL
        ));

        // Calling cursor reconfigure with an invalid config string fails
        assert_ok!(sess.create("table:foo", ""));
        let cur = assert_ok!(sess.open_cursor("table:foo"));
        assert!(matches!(
            cur.reconfigure("bogus"),
            Err(Error {
                code,
                message,
            })
            if message == "Invalid argument" && code == libc::EINVAL
        ));

        // Reconfigure with valid args is successful
        assert_ok!(sess.reconfigure("cache_max_wait_ms=12"));
        assert_ok!(conn.reconfigure("eviction_target=75"));
        assert_ok!(cur.reconfigure("append=true"));
    }
}
