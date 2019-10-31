use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;


const FUSE_USE_VERSION: u32 = 34;


fn main() {
    let fuse = pkg_config::Config::new()
        .atleast_version("3.7.0")
        .probe("fuse3")
        .unwrap();

    let fuse_header = find_fuse_header(&fuse.include_paths).unwrap();
    let fuse_header = fuse_header.to_str().unwrap();

    println!("cargo:rerun-if-changed={}", fuse_header);

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let fuse_header = generate_fuse_header(&out_path, fuse_header);
    let fuse_header = fuse_header.to_str().unwrap();

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

    let mut file = File::create(&path).unwrap();

    file.write_all(content.as_bytes()).unwrap();

    path
}
