#![allow(unused_variables)]

use std::{ env, process };

use libc;
use libc::{ c_int, c_char };

use libfuse_sys;
use libfuse_sys::fuse;


fn main() {
    if let Err(e) = libfuse_sys::fuse_main(env::args(), Hello {
        filename: String::from("hello"),
        contents: String::from("hello, world!"),
    }) {
        process::exit(e);
    }
}


struct Hello {
    filename: String,
    contents: String,
}

impl libfuse_sys::Operations for Hello {
    fn init(&mut self, conn: &mut fuse::fuse_conn_info, cfg: &mut fuse::fuse_config) {
        cfg.kernel_cache = 1
    }

    fn getattr(&mut self,
        path: &str,
        stbuf: &mut fuse::stat,
        fi: &mut fuse::fuse_file_info) -> c_int
    {
        stbuf.clear();

        if path == "/" {
            stbuf.st_mode = libc::S_IFDIR | 0o755;
            stbuf.st_nlink = 2;
        } else if &path[1..] == self.filename {
            stbuf.st_mode = libc::S_IFREG | 0444;
            stbuf.st_nlink = 1;
            stbuf.st_size = self.contents.len() as fuse::off_t;
        } else {
            return -libc::ENOENT;
        }

        0
    }

    fn readdir(&mut self,
        path: &str,
        filler: &mut dyn FnMut(
            &str, Option<&fuse::stat>, fuse::off_t, fuse::fuse_fill_dir_flags) -> c_int,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info,
        flags:fuse::fuse_readdir_flags) -> c_int
    {
        if path != "/" {
            return -libc::ENOENT;
        }

        filler(".", None, 0, 0);
        filler("..", None, 0, 0);
        filler(&self.filename, None, 0, 0);

        0
    }

    fn open(&mut self, path: &str, fi: &mut fuse::fuse_file_info) -> c_int {
        if &path[1..] != self.filename {
            return -libc::ENOENT;
        }

        if (fi.flags & libc::O_ACCMODE) != libc::O_RDONLY {
            return -libc::EACCES;
        }

        0
    }

    fn read(&mut self,
        path: &str,
        buf: &mut [c_char],
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info) -> c_int
    {
        if &path[1..] != self.filename {
            return -libc::ENOENT;
        }

        let mut size = buf.len() as fuse::off_t;
        let len = self.contents.len() as fuse::off_t;

        if offset < len {
            if offset + size > len {
                size = len - offset;
            }

            let ptr = (&self.contents[offset as usize ..]).as_ptr() as *const c_char;

            unsafe {
                std::ptr::copy(ptr, buf.as_mut_ptr(), size as usize);
            }

            return size as c_int;
        }

        0
    }
}
