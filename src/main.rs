use wiredtiger::Connection;

fn main() {
    let conn = Connection::open("/tmp/wttest", "create").unwrap();
    let session = conn.open_session().unwrap();
    let _ = session.create("table:mytable", "key_format=S,value_format=S");
    let cursor = session.open_cursor("table:mytable").unwrap();
    let _ = cursor.set("tyler", "brock");
    let _ = cursor.set("mike", "obrien");
    println!("tyler: {:?}", cursor.search("tyler").unwrap());
    println!("mike: {:?}", cursor.search("mike").unwrap());
}
