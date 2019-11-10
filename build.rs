use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use unwrap::unwrap;


const FUSE_USE_VERSION: u32 = 34;


fn main() {
    let fuse = unwrap!(pkg_config::Config::new()
        .atleast_version("3.8.0")
        .probe("fuse3"));

    let fuse_header = unwrap!(find_fuse_header(&fuse.include_paths));
    let fuse_header = unwrap!(fuse_header.to_str());

    println!("cargo:rerun-if-changed={}", fuse_header);

    let out_path = PathBuf::from(unwrap!(env::var("OUT_DIR")));

    let fuse_header = generate_fuse_header(&out_path, fuse_header);
    let fuse_header = unwrap!(fuse_header.to_str());

    // install rustfmt(rustup component add rustfmt) to get formated bindings
    let bindings = bindgen::Builder::default()
        .header(fuse_header)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn find_fuse_header(paths: &Vec<PathBuf>) -> Option<PathBuf> {
    for path in paths {
        let header = path.join("fuse.h");
        if header.exists() {
            return Some(header);
        }
    }

    None
}

fn generate_fuse_header(out: &PathBuf, fuse_header: &str) -> PathBuf {
    let content = format!("\
#define FUSE_USE_VERSION {}
#include \"{}\"",
        FUSE_USE_VERSION, fuse_header);

    let path = out.join("fuse.h");

    let mut file = unwrap!(File::create(&path));

    unwrap!(file.write_all(content.as_bytes()));

    path
}
