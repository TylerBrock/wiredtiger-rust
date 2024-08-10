use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::{create_dir, remove_dir_all};

const GITHUB_WT_TAGS_URI: &str = "https://github.com/wiredtiger/wiredtiger/archive/refs/tags";
const WT_VERSION: &str = "11.2.0";
const THIRD_PARTY_DIR: &str = "third_party";

fn download_source() {
    remove_dir_all(THIRD_PARTY_DIR);
    create_dir(THIRD_PARTY_DIR).expect("Failed to create third_party directory");

    let uri = format!("{GITHUB_WT_TAGS_URI}/{WT_VERSION}.tar.gz");
    let dest = format!("./{THIRD_PARTY_DIR}");
    let output = Command::new("wget")
        .arg(uri)
        .arg("-P")
        .arg(dest)
        .output()
        .expect("Failed to download source");

    // Check if the command was successful
    if output.status.success() {
        println!("Source downloaded successfully");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("Failed to download source: {}", error_message);
    }
}

fn extract_source() {
    let output = Command::new("tar")
        .arg("-xvf")
        .arg(format!("{THIRD_PARTY_DIR}/{WT_VERSION}.tar.gz"))
        .arg("-C")
        .arg(THIRD_PARTY_DIR)
        .output()
        .expect("Failed to extract source");

    // Check if the command was successful
    if output.status.success() {
        println!("Source extracted successfully");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("Failed to extract source: {}", error_message);
    }
}

fn build_wt() {
    let src_dir = format!("{THIRD_PARTY_DIR}/wiredtiger-{WT_VERSION}");
    let out_dir = format!("{src_dir}/build");
    let output = Command::new("cmake")
        .arg("-DENABLE_STATIC=1")
        .arg("-S")
        .arg(src_dir)
        .arg("-b")
        .arg(out_dir)
        .output()
        .expect("Failed to build WiredTiger");
    if output.status.success() {
        println!("Source downloaded successfully");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("Failed to download source: {}", error_message);
    }
    let output = Command::new("cmake")
        .arg("--build")
        .arg(".")
        .output()
        .expect("Failed to build WiredTiger");
    if output.status.success() {
        println!("Source downloaded successfully");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("Failed to download source: {}", error_message);
    }
}

fn cleanup() {
    let path =format!("{THIRD_PARTY_DIR}/wiredtiger-{WT_VERSION}.tar.gz");
    remove_dir_all(path).expect("Failed to cleanup"); 
}

fn main() {
    download_source();
    extract_source();
    build_wt();
    cleanup();

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=./third_party/wiredtiger-11.2.0/build");

    // Tell cargo to tell rustc to statically link with the wiredtiger library.
    // This requires that WT was configured with the -DENABLE_STATIC=1 option to cmake.
    println!("cargo:rustc-link-lib=static=wiredtiger");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
