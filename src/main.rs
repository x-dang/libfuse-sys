use std::{ env, process };
use libfuse_sys;

fn main() {
    if let Err(e) = libfuse_sys::fuse_main(env::args(), Foo()) {
        process::exit(e);
    }
}

struct Foo();

impl libfuse_sys::operations::FuseOperations for Foo { }
