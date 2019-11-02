use std::sync::Once;
use std::ffi::{ CStr, CString };
use std::convert::TryInto;

use libc::{ c_int, c_uint, c_char, c_void };
use libc::ENOSYS;

use unwrap::unwrap;

use crate::{ fuse, Neg, neg };


macro_rules! op_method {
    ( $method:ident; $( $arg:ident : $T:ty ),* ) => {
        fn $method(&mut self, $( $arg: $T, )*) -> Result<(), Neg> { Err(neg!(-ENOSYS)) }
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

#[allow(unused_variables)]
pub trait Operations {
    op_method! { getattr;
        path: &str,
        stbuf: &mut fuse::stat,
        fi: Option<&mut fuse::fuse_file_info>
    }

    fn readlink(&mut self, path: &str) -> Result<String, Neg> { Err(neg!(-ENOSYS)) }

    op_method! { mknod  ; path: &str, mode: fuse::mode_t, rdev: fuse::dev_t }
    op_method! { mkdir  ; path: &str, mode: fuse::mode_t }
    op_method! { unlink ; path: &str }
    op_method! { rmdir  ; path: &str }
    op_method! { symlink; from: &str, to: &str }
    op_method! { rename ; from: &str, to: &str, flags: c_uint }
    op_method! { link   ; from: &str, to: &str }
    op_method! { chmod  ; path: &str, mode: fuse::mode_t, fi: Option<&mut fuse::fuse_file_info> }

    op_method! { chown;
        path: &str,
        uid: fuse::uid_t,
        gid: fuse::gid_t,
        fi: Option<&mut fuse::fuse_file_info>
    }

    op_method! { truncate; path: &str, size: fuse::off_t, fi: Option<&mut fuse::fuse_file_info> }
    op_method! { open    ; path: &str, fi: &mut fuse::fuse_file_info }

    op_method! { read;
        path: &str,
        filler: &mut dyn FnMut(&[u8]) -> Result<usize, ()>,
        size: usize,
        offset: fuse::off_t,
        fi: Option<&mut fuse::fuse_file_info>
    }

    fn write(&mut self,
        path: &str,
        buf: &[u8],
        offset: fuse::off_t,
        fi: Option<&mut fuse::fuse_file_info>) -> Result<usize, Neg> { Err(neg!(-ENOSYS)) }

    op_method! { statfs; path: &str, stbuf: &mut fuse::statvfs }
    op_method! { flush ; path: &str, fi: &mut fuse::fuse_file_info }

    fn release(&mut self, path: &str, fi: &mut fuse::fuse_file_info) { }

    op_method! { fsync   ; path: &str, isdatasync: bool, fi: &mut fuse::fuse_file_info }
    op_method! { setxattr; path: &str, name: &str, value: &[u8], flags: c_int }

    op_method! { getxattr;
        path: &str,
        name: &str,
        filler: &mut dyn FnMut(&[u8]) -> Result<usize, ()>,
        size: usize
    }

    op_method! { readdir;
        path: &str,
        filler: &dyn Fn(
            &str, Option<&fuse::stat>, fuse::off_t, fuse::fuse_fill_dir_flags) -> Result<(), ()>,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info,
        flags: fuse::fuse_readdir_flags
    }

    fn init(&mut self, info: &mut fuse::fuse_conn_info, conf: &mut fuse::fuse_config) { }
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
        unwrap!(USER_OPERATIONS.as_mut()).$method( $( $arg, )* )
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

macro_rules! filler_mut {
    ( $buf:ident, $size:expr, $index:ident ) => {
        &mut |src| {
            let len = src.len();

            if len <= $size - $index {
                $buf.add($index).copy_from_nonoverlapping(src.as_ptr().cast(), len);
                $index += len;

                Ok($index)
            } else {
                Err(())
            }
        }
    }
}

unsafe extern "C" fn getattr(
    path: *const c_char,
    stbuf: *mut fuse::stat,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(getattr, ptr_str!(path), ptr_mut!(stbuf), fi.as_mut()))
}

unsafe extern "C" fn readlink(path: *const c_char, buf: *mut c_char, size: usize) -> c_int {
    match op!(readlink, ptr_str!(path)) {
        Err(e) => e.get(),
        Ok(s) => {
            let s = unwrap!(CString::new(s));
            let s = s.as_bytes();

            let size = s.len().min(size - 1);
            buf.copy_from_nonoverlapping(s.as_ptr().cast(), size);
            *buf.add(size) = 0;

            0
        }
    }
}

unsafe extern "C" fn mknod(path: *const c_char, mode: fuse::mode_t, rdev: fuse::dev_t) -> c_int {
    op_result!(op!(mknod, ptr_str!(path), mode, rdev))
}

unsafe extern "C" fn mkdir(path: *const c_char, mode: fuse::mode_t,) -> c_int {
    op_result!(op!(mkdir, ptr_str!(path), mode))
}

unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    op_result!(op!(unlink, ptr_str!(path)))
}

unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    op_result!(op!(rmdir, ptr_str!(path)))
}

unsafe extern "C" fn symlink(from: *const c_char, to: *const c_char) -> c_int {
    op_result!(op!(symlink, ptr_str!(from), ptr_str!(to)))
}

unsafe extern "C" fn rename(from: *const c_char, to: *const c_char, flags: c_uint) -> c_int {
    op_result!(op!(rename, ptr_str!(from), ptr_str!(to), flags))
}

unsafe extern "C" fn link(from: *const c_char, to: *const c_char) -> c_int {
    op_result!(op!(link, ptr_str!(from), ptr_str!(to)))
}

unsafe extern "C" fn chmod(
    path: *const c_char,
    mode: fuse::mode_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(chmod, ptr_str!(path), mode, fi.as_mut()))
}

unsafe extern "C" fn chown(
    path: *const c_char,
    uid: fuse::uid_t,
    gid: fuse::gid_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(chown, ptr_str!(path), uid, gid, fi.as_mut()))
}

unsafe extern "C" fn truncate(
    path: *const c_char,
    size: fuse::off_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(truncate, ptr_str!(path), size, fi.as_mut()))
}

unsafe extern "C" fn open(path: *const c_char, fi: *mut fuse::fuse_file_info) -> c_int {
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

    match op!(read, ptr_str!(path), filler_mut!(buf, size, index), size, offset, fi.as_mut()) {
        Ok(_) => unwrap!(index.try_into()),
        Err(e) => e.get(),
    }
}

unsafe extern "C" fn write(
    path: *const c_char,
    buf: *const c_char,
    size: usize,
    offset: fuse::off_t,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    let res = op!(write,
        ptr_str!(path),
        std::slice::from_raw_parts(buf.cast(), size),
        offset,
        fi.as_mut());

    match res {
        Ok(x) => unwrap!(x.try_into()),
        Err(e) => e.get(),
    }
}

unsafe extern "C" fn statfs(path: *const c_char, stbuf: *mut fuse::statvfs) -> c_int {
    op_result!(op!(statfs, ptr_str!(path), ptr_mut!(stbuf)))
}

unsafe extern "C" fn flush(path: *const c_char, fi: *mut fuse::fuse_file_info) -> c_int {
    op_result!(op!(flush, ptr_str!(path), ptr_mut!(fi)))
}

unsafe extern "C" fn release(path: *const c_char, fi: *mut fuse::fuse_file_info) -> c_int {
    op!(release, ptr_str!(path), ptr_mut!(fi));

    0
}

unsafe extern "C" fn fsync(
    path: *const c_char,
    isdatasync: c_int,
    fi: *mut fuse::fuse_file_info) -> c_int
{
    op_result!(op!(fsync, ptr_str!(path), isdatasync != 0, ptr_mut!(fi)))
}

unsafe extern "C" fn setxattr(
    path: *const c_char,
    name: *const c_char,
    value: *const c_char,
    size: usize,
    flags: c_int) -> c_int
{
    op_result!(op!(setxattr,
        ptr_str!(path),
        ptr_str!(name),
        std::slice::from_raw_parts(value.cast(), size),
        flags))
}

unsafe extern "C" fn getxattr(
    path: *const c_char,
    name: *const c_char,
    value: *mut c_char,
    size: usize) -> c_int
{
    let mut index = 0usize;

    match op!(getxattr, ptr_str!(path), ptr_str!(name), filler_mut!(value, size, index), size) {
        Ok(_) => unwrap!(index.try_into()),
        Err(e) => e.get()
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
    let filler = unwrap!(filler);

    op_result!(op!(readdir,
        ptr_str!(path),
        &|name, stbuf, offset, flags| {
            let name = unwrap!(CString::new(name));

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
        flags))
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
        mknod: Some(mknod),
        mkdir: Some(mkdir),
        unlink: Some(unlink),
        rmdir: Some(rmdir),
        symlink: Some(symlink),
        rename: Some(rename),
        link: Some(link),
        chmod: Some(chmod),
        chown: Some(chown),
        truncate: Some(truncate),
        open: Some(open),
        read: Some(read),
        write: Some(write),
        statfs: Some(statfs),
        flush: Some(flush),
        release: Some(release),
        fsync: Some(fsync),
        setxattr: Some(setxattr),
        getxattr: Some(getxattr),
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
