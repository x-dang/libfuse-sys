#![allow(unused_variables)]


use super::fuse;

use std::sync::Once;
use std::os::raw::{ c_int, c_char, c_void };


pub trait FuseOperations {
    fn getattr(&mut self) -> i32 {
        0
    }

    fn open(&mut self) -> i32 {
        0
    }

    fn read(&mut self) -> i32 {
        0
    }

    fn readdir(&mut self) -> i32 {
        0
    }
}


static mut USER_OPERATIONS: Option<Box<dyn FuseOperations>> = None;
static INIT: Once = Once::new();

pub fn set_operations<T: 'static + FuseOperations>(ops: T) {
    unsafe {
        INIT.call_once(|| USER_OPERATIONS = Some(Box::new(ops)));
    }
}


unsafe extern "C" fn ops_getattr(
    path: *const c_char,
    stbuf: *mut fuse::stat,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    USER_OPERATIONS.as_mut().unwrap().getattr()
}

unsafe extern "C" fn ops_open(path: *const c_char, fi: *mut fuse::fuse_file_info) -> c_int {
    USER_OPERATIONS.as_mut().unwrap().open()
}

unsafe extern "C" fn ops_read(
    path: *const c_char,
    buf: *mut c_char,
    size: usize,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    USER_OPERATIONS.as_mut().unwrap().read()
}

unsafe extern "C" fn ops_readdir(
    path: *const c_char,
    buf: *mut c_void,
    filler: fuse::fuse_fill_dir_t,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info,
    flags: fuse::fuse_readdir_flags) -> c_int
{
    USER_OPERATIONS.as_mut().unwrap().readdir()
}

impl fuse::fuse_operations {
    pub fn new() -> fuse::fuse_operations {
        fuse::fuse_operations {
            getattr: Some(ops_getattr),
            readlink: None,
            mknod: None,
            mkdir: None,
            unlink: None,
            rmdir: None,
            symlink: None,
            rename: None,
            link: None,
            chmod: None,
            chown: None,
            truncate: None,
            open: Some(ops_open),
            read: Some(ops_read),
            write: None,
            statfs: None,
            flush: None,
            release: None,
            fsync: None,
            setxattr: None,
            getxattr: None,
            listxattr: None,
            removexattr: None,
            opendir: None,
            readdir: Some(ops_readdir),
            releasedir: None,
            fsyncdir: None,
            init: None,
            destroy: None,
            access: None,
            create: None,
            lock: None,
            utimens: None,
            bmap: None,
            ioctl: None,
            poll: None,
            write_buf: None,
            read_buf: None,
            flock: None,
            fallocate: None,
            copy_file_range: None,
        }
    }
}
