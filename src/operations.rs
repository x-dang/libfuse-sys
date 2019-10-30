use super::fuse;

use std::sync::Once;
use std::ffi::{ CStr, CString };

use libc;
use libc::{ c_int, c_char, c_void };


#[allow(unused_variables)]
pub trait Operations {
    fn getattr(&mut self,
        path: &str,
        stbuf: &mut fuse::stat,
        fi: Option<&mut fuse::fuse_file_info>) -> c_int { -libc::ENOSYS }

    fn open(&mut self,
        path: &str,
        fi: &mut fuse::fuse_file_info) -> c_int { -libc::ENOSYS }

    fn read(&mut self,
        path: &str,
        filler: &mut dyn FnMut(&[u8]) -> Result<usize, ()>,
        size: usize,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info) -> c_int { -libc::ENOSYS }

    fn readdir(&mut self,
        path: &str,
        filler: &dyn Fn(&str, Option<&fuse::stat>, fuse::off_t, fuse::fuse_fill_dir_flags)
            -> Result<c_int, c_int>,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info,
        flags: fuse::fuse_readdir_flags) -> c_int { -libc::ENOSYS }

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


unsafe fn str_from_ptr<'a>(ptr: *const c_char) -> &'a str {
    CStr::from_ptr(ptr).to_str().unwrap()
}


unsafe extern "C" fn getattr(
    path: *const c_char,
    stbuf: *mut fuse::stat,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op!(getattr,
        str_from_ptr(path),
        stbuf.as_mut().unwrap(),
        fi.as_mut())
}

unsafe extern "C" fn open(
    path: *const c_char,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op!(open,
        str_from_ptr(path),
        fi.as_mut().unwrap())
}

unsafe extern "C" fn read(
    path: *const c_char,
    buf: *mut c_char,
    size: usize,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    let mut index = 0usize;

    op!(read,
        str_from_ptr(path),
        &mut |src| {
            if src.len() > (size - index) {
                Err(())
            } else {
                buf.add(index).copy_from_nonoverlapping(src.as_ptr().cast(), src.len());
                index += src.len();
                Ok(index)
            }
        },
        size,
        offset,
        fi.as_mut().unwrap())
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

    op!(readdir,
        str_from_ptr(path),
        &|name, stbuf, offset, flags| {
            let name = CString::new(name).unwrap();

            let stbuf = if let Some(x) = stbuf {
                x
            } else {
                std::ptr::null()
            };

            let res = filler(buf, name.as_ptr(), stbuf, offset, flags);

            if res == 0 {
                Ok(0)
            } else {
                Err(res)
            }
        },
        offset,
        fi.as_mut().unwrap(),
        flags)
}

unsafe extern "C" fn init(
    info: *mut fuse::fuse_conn_info,
    conf: *mut fuse::fuse_config) -> *mut c_void
{
    op!(init,
        info.as_mut().unwrap(),
        conf.as_mut().unwrap());

    std::ptr::null_mut()
}

fn fuse_operations_new() -> fuse::fuse_operations {
    fuse::fuse_operations {
        getattr: Some(getattr),
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
