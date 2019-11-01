use std::sync::Once;
use std::ffi::{ CStr, CString };
use std::convert::TryInto;

use libc::{ c_int, c_char, c_void };
use libc::ENOSYS;

use crate::{ fuse, Neg, neg };


macro_rules! op_method {
    ( $method:ident; $( $arg:ident : $T:ty ),* ) => {
        fn $method(&mut self, $( $arg: $T, )*) -> Result<(), Neg> { Err(neg!(-ENOSYS)) }
    };
}

#[allow(unused_variables)]
pub trait Operations {
    op_method! { getattr;
        path: &str,
        stbuf: &mut fuse::stat,
        fi: Option<&mut fuse::fuse_file_info>
    }

    fn readlink(&mut self,
        path: &str) -> Result<String, Neg> { Err(neg!(-ENOSYS)) }

    op_method! { open; path: &str, fi: &mut fuse::fuse_file_info }

    fn read(&mut self,
        path: &str,
        filler: &mut dyn FnMut(&[u8]) -> Result<usize, ()>,
        size: usize,
        offset: fuse::off_t,
        fi: Option<&mut fuse::fuse_file_info>) -> Result<usize, Neg> { Err(neg!(-ENOSYS)) }

    op_method! { readdir;
        path: &str,
        filler: &dyn Fn(
            &str, Option<&fuse::stat>, fuse::off_t, fuse::fuse_fill_dir_flags) -> Result<(), ()>,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info,
        flags: fuse::fuse_readdir_flags
    }

    fn init(&mut self,
        info: &mut fuse::fuse_conn_info,
        conf: &mut fuse::fuse_config) { }
}


static mut USER_OPERATIONS: Option<Box<dyn Operations>> = None;
static INIT: Once = Once::new();

pub fn set_operations<T: 'static + Operations>(ops: T) -> fuse::fuse_operations {
    unsafe {
        INIT.call_once(|| USER_OPERATIONS = Some(Box::new(ops)));
    }

    fuse_operations_new()
}

macro_rules! op {
    ( $method:ident, $( $arg:expr ),* ) => {
        USER_OPERATIONS.as_mut().unwrap().$method( $( $arg, )* )
    };
}


macro_rules! ptr_str {
    ( $ptr:expr ) => {
        match CStr::from_ptr($ptr).to_str() {
            Ok(x) => x,
            Err(e) => panic!("convert '*const c_char' to '&str' failed: {:?}", e),
        }
    };
}

macro_rules! ptr_mut {
    ( $ptr:expr ) => {
        match $ptr.as_mut() {
            Some(x) => x,
            None => panic!("try to convert a null ptr to mutable reference"),
        }
    };
}


macro_rules! op_result {
    ( $op:expr ) => {
        if let Err(e) = $op {
            e.get()
        } else {
            0
        }
    };
}

unsafe extern "C" fn getattr(
    path: *const c_char,
    stbuf: *mut fuse::stat,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(getattr, ptr_str!(path), ptr_mut!(stbuf), fi.as_mut()))
}

unsafe extern "C" fn readlink(
    path: *const c_char,
    buf: *mut c_char,
    size: usize) -> c_int
{
    match op!(readlink, ptr_str!(path)) {
        Err(e) => e.get(),
        Ok(s) => {
            let s = CString::new(s).unwrap();
            let s = s.as_bytes();

            let size = s.len().min(size - 1);
            buf.copy_from_nonoverlapping(s.as_ptr().cast(), size);
            *buf.add(size) = 0;

            0
        }
    }
}

unsafe extern "C" fn open(
    path: *const c_char,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(open, ptr_str!(path), ptr_mut!(fi)))
}

unsafe extern "C" fn read(
    path: *const c_char,
    buf: *mut c_char,
    size: usize,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    let mut index = 0usize;

    let res = op!(read,
        ptr_str!(path),
        &mut |src| {
            let len = src.len();

            if len <= size - index {
                buf.add(index).copy_from_nonoverlapping(src.as_ptr().cast(), len);
                index += len;

                Ok(index)
            } else {
                Err(())
            }
        },
        size,
        offset,
        fi.as_mut());

    match res {
        Ok(x) => x.try_into().unwrap(),
        Err(e) => e.get(),
    }
}

unsafe extern "C" fn readdir(
    path: *const c_char,
    buf: *mut c_void,
    filler: fuse::fuse_fill_dir_t,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info,
    flags: fuse::fuse_readdir_flags) -> c_int
{
    let filler = filler.unwrap();

    op_result!(op!(readdir,
        ptr_str!(path),
        &|name, stbuf, offset, flags| {
            let name = CString::new(name).unwrap();

            let stbuf = if let Some(x) = stbuf {
                x
            } else {
                std::ptr::null()
            };

            let res = filler(buf, name.as_ptr(), stbuf, offset, flags);

            if res == 0 {
                Ok(())
            } else {
                assert_eq!(res, 1);
                Err(())
            }
        },
        offset,
        ptr_mut!(fi),
        flags)
    )
}

unsafe extern "C" fn init(
    info: *mut fuse::fuse_conn_info,
    conf: *mut fuse::fuse_config) -> *mut c_void
{
    op!(init, ptr_mut!(info), ptr_mut!(conf));

    std::ptr::null_mut()
}

fn fuse_operations_new() -> fuse::fuse_operations {
    fuse::fuse_operations {
        getattr: Some(getattr),
        readlink: Some(readlink),
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
        open: Some(open),
        read: Some(read),
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
        readdir: Some(readdir),
        releasedir: None,
        fsyncdir: None,
        init: Some(init),
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
