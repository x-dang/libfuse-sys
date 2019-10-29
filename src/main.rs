use std::{ env, process };
use libfuse_sys;

fn main() {
    if let Err(e) = libfuse_sys::fuse_main(env::args(), Hello()) {
        process::exit(e);
    }
}

struct Hello();

impl libfuse_sys::Operations for Hello { }
