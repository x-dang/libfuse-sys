use std::env;
use libfuse_sys;

fn main() {
    libfuse_sys::fuse_main(env::args(), Foo());
}

struct Foo();

impl libfuse_sys::operations::FuseOperations for Foo { }
