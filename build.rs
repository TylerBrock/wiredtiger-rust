use std::env;
use std::path::PathBuf;

fn main() {
    let (include_dir, lib_dir) = find_wiredtiger();

    if let Some(lib_dir) = lib_dir {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=wiredtiger");
        println!("cargo:rustc-link-lib=dylib=pthread");
    }
    // When found via pkg-config, linking flags are emitted automatically.

    let header = include_dir.join("wiredtiger.h");
    println!("cargo:rerun-if-changed={}", header.display());
    println!("cargo:rerun-if-env-changed=WIREDTIGER_DIR");

    let bindings = bindgen::Builder::default()
        .header(header.to_str().expect("non-UTF-8 header path"))
        .allowlist_type("WT_.*|__wt_.*")
        .allowlist_function("wiredtiger_.*")
        .allowlist_var("WT_.*")
        .generate_comments(false)
        .generate()
        .expect("failed to generate wiredtiger bindings");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("failed to write bindings");
}

/// Returns (include_dir, optional_lib_dir).
/// When found via pkg-config, lib_dir is None because pkg-config emits link flags directly.
fn find_wiredtiger() -> (PathBuf, Option<PathBuf>) {
    // 1. Explicit override via environment variable.
    if let Ok(dir) = env::var("WIREDTIGER_DIR") {
        let base = PathBuf::from(dir);
        return (base.join("include"), Some(base));
    }

    // 2. pkg-config (the standard way on Linux/macOS with system-installed WiredTiger).
    match pkg_config::Config::new()
        .atleast_version("12.0.0")
        .probe("wiredtiger")
    {
        Ok(lib) => {
            let include_dir = lib
                .include_paths
                .into_iter()
                .find(|p| p.join("wiredtiger.h").exists())
                .expect("pkg-config found wiredtiger but wiredtiger.h is missing from include paths");
            return (include_dir, None);
        }
        Err(e) => {
            // Not fatal — fall through to the next heuristic.
            println!("cargo:warning=pkg-config could not find wiredtiger: {e}");
        }
    }

    // 3. Sibling build directory (useful during local development).
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let sibling = manifest.parent().unwrap().join("wiredtiger").join("build");
    if sibling.join("libwiredtiger.a").exists() {
        return (sibling.join("include"), Some(sibling));
    }

    panic!(
        "\n\nCould not find WiredTiger 12.x.\n\
         \n\
         Options:\n\
         \n\
         1. Install via your system package manager (if WiredTiger 12 packages are available).\n\
         \n\
         2. Build from source:\n\
              git clone https://github.com/wiredtiger/wiredtiger\n\
              cd wiredtiger && cmake -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build -j$(nproc)\n\
         \n\
         3. Set WIREDTIGER_DIR to the build directory:\n\
              export WIREDTIGER_DIR=/path/to/wiredtiger/build\n\
              cargo build\n\
         \n\
         The build directory must contain libwiredtiger.a and include/wiredtiger.h.\n"
    );
}
