# wiredtiger

Safe Rust bindings for the [WiredTiger](https://github.com/wiredtiger/wiredtiger) storage engine, targeting WiredTiger 12.x.

## Usage

```toml
[dependencies]
wiredtiger = "12"
```

## Prerequisites

WiredTiger 12 must be installed or built before compiling this crate.

### Build from source (recommended)

```sh
git clone https://github.com/wiredtiger/wiredtiger
cd wiredtiger
cmake -B build -DCMAKE_BUILD_TYPE=Release -DENABLE_STATIC=1
cmake --build build -j$(nproc)
export WIREDTIGER_DIR=$(pwd)/build
```

### System install (when packages are available)

```sh
# Debian/Ubuntu (if WT 12 packages exist)
apt-get install libwiredtiger-dev
```

If WiredTiger is installed to a non-standard prefix, set `WIREDTIGER_DIR` to the
directory containing `libwiredtiger.a` and `include/wiredtiger.h`.

## Example

```rust
use wiredtiger::Connection;

let conn = Connection::open("/tmp/mydb", "create,cache_size=256MB")?;
let sess = conn.open_session()?;

sess.create("table:test", "key_format=u,value_format=u")?;

let mut cur = sess.open_cursor("table:test", "")?;
cur.set_key(b"hello");
cur.set_value(b"world");
cur.insert()?;

cur.set_key(b"hello");
cur.search()?;
let val = cur.get_value()?;
assert_eq!(val, b"world");
```

## License

Apache-2.0. Note: WiredTiger itself is GPL-2.0-or-later (or commercially licensed from MongoDB).
