#![allow(unused_variables)]


use super::fuse;

use std::sync::Once;
use std::os::raw::{ c_int, c_char, c_void };

use std::ffi::{ CStr, CString };


pub trait Operations {
    fn getattr(&mut self,
        path: &str,
        stbuf: &mut fuse::stat,
        fi: &mut fuse::fuse_file_info) -> c_int { -1 }

    fn open(&mut self, path: &str, fi: &mut fuse::fuse_file_info) -> c_int { -1 }

    fn read(&mut self,
        path: &str,
        buf: &mut [c_char],
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info) -> c_int { -1 }

    fn readdir(&mut self,
        path: &str,
        filler: &mut dyn FnMut(
            &str, Option<&fuse::stat>, fuse::off_t, fuse::fuse_fill_dir_flags) -> c_int,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info,
        flags:fuse::fuse_readdir_flags) -> c_int { -1 }

    fn init(&mut self, conn: &mut fuse::fuse_conn_info, cfg: &mut fuse::fuse_config) { }
}


static mut USER_OPERATIONS: Option<Box<dyn Operations>> = None;
static INIT: Once = Once::new();

pub fn set_operations<T: 'static + Operations>(ops: T) {
    unsafe {
        INIT.call_once(|| USER_OPERATIONS = Some(Box::new(ops)));
    }
}


unsafe fn str_from_ptr<'a>(ptr: *const c_char) -> &'a str {
    CStr::from_ptr(ptr).to_str().unwrap()
}


unsafe extern "C" fn ops_getattr(
    path: *const c_char,
    stbuf: *mut fuse::stat,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    USER_OPERATIONS.as_mut().unwrap().getattr(str_from_ptr(path), &mut *stbuf, &mut *fi)
}

unsafe extern "C" fn ops_open(path: *const c_char, fi: *mut fuse::fuse_file_info) -> c_int {
    USER_OPERATIONS.as_mut().unwrap().open(str_from_ptr(path), &mut *fi)
}

unsafe extern "C" fn ops_read(
    path: *const c_char,
    buf: *mut c_char,
    size: usize,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    USER_OPERATIONS.as_mut().unwrap().read(
        str_from_ptr(path),
        std::slice::from_raw_parts_mut(buf, size),
        offset,
        &mut *fi)
}

unsafe extern "C" fn ops_readdir(
    path: *const c_char,
    buf: *mut c_void,
    filler: fuse::fuse_fill_dir_t,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info,
    flags: fuse::fuse_readdir_flags) -> c_int
{
    let filler = filler.unwrap();

    USER_OPERATIONS.as_mut().unwrap().readdir(
        str_from_ptr(path),
        &mut |name, stbuf, off, flags| {
            let name = CString::new(name).unwrap();

            let stbuf = if let Some(x) = stbuf {
                x
            } else {
                std::ptr::null()
            };

            filler(buf, name.as_ptr(), stbuf, off, flags)
        },
        offset,
        &mut *fi,
        flags)
}

unsafe extern "C" fn ops_init(
    conn: *mut fuse::fuse_conn_info,
    cfg: *mut fuse::fuse_config) -> *mut c_void
{
    USER_OPERATIONS.as_mut().unwrap().init(&mut *conn, &mut *cfg);

    std::ptr::null_mut()
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
            init: Some(ops_init),
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
