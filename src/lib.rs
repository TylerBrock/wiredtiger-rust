mod raw {
    #![allow(
        non_upper_case_globals, non_camel_case_types, non_snake_case,
        dead_code, unused_imports, clippy::all
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod error;
pub mod connection;
pub mod session;
pub mod cursor;

pub use error::{Error, Result};
pub use connection::Connection;
pub use session::Session;
pub use cursor::Cursor;
