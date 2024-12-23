use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn build_wt() -> std::io::Result<()> {
    let wt_dir = "wiredtiger";
    let build_dir = format!("{wt_dir}/build");
    Command::new("cmake")
        .arg("-DENABLE_STATIC=1")
        .arg("-S")
        .arg(wt_dir)
        .arg("-B")
        .arg(&build_dir)
        .output()?;
    Command::new("cmake")
        .arg("--build")
        .arg(&build_dir)
        .arg("-j16")
        .output()?;
    Ok(())
}

fn bindgen_wt() {
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("wiredtiger/build/include/wiredtiger.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}

fn main() {
    if !Path::new("wiredtiger/LICENSE").exists() {
        update_submodules();
    }
    build_wt().expect("Failed to build wiredtiger");

    bindgen_wt();

    // Tell cargo to look for shared libraries in the specified directory.
    // Note that this search path is relative to the repo root.
    println!("cargo:rustc-link-search=crates/wiredtiger/wiredtiger-sys/wiredtiger/build");

    // Tell cargo to tell rustc to statically link with the wiredtiger library.
    // This requires that WT was configured with the -DENABLE_STATIC=1 option to cmake.
    println!("cargo:rustc-link-lib=static=wiredtiger");
}

fn update_submodules() {
    let program = "git";
    let dir = "../";
    let args = ["submodule", "update", "--init"];
    println!(
        "Running command: \"{} {}\" in dir: {}",
        program,
        args.join(" "),
        dir
    );
    let ret = Command::new(program).current_dir(dir).args(args).status();

    match ret.map(|status| (status.success(), status.code())) {
        Ok((true, _)) => (),
        Ok((false, Some(c))) => panic!("Command failed with error code {}", c),
        Ok((false, None)) => panic!("Command got killed"),
        Err(e) => panic!("Command failed with error: {}", e),
    }
}
