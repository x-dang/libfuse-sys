#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod fuse_bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}


pub use fuse_bindings::*;


impl stat {
    pub fn clear(&mut self) {
        unsafe { std::ptr::write_bytes(self, 0, 1); }
    }
}
