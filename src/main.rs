use std::{ env, process };

use unwrap::unwrap;

use libfuse_sys::{ fuse, Neg, neg };


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

#[allow(unused_variables)]
impl libfuse_sys::Operations for Hello {
    fn init(&mut self,
        info: &mut fuse::fuse_conn_info,
        conf: &mut fuse::fuse_config)
    {
        conf.kernel_cache = 1
    }

    fn getattr(&mut self,
        path: &str,
        stbuf: &mut fuse::stat,
        fi: Option<&mut fuse::fuse_file_info>) -> Result<(), Neg>
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
            return Err(neg!(-libc::ENOENT));
        }

        Ok(())
    }

    fn readdir(&mut self,
        path: &str,
        filler: &dyn Fn(
            &str, Option<&fuse::stat>, fuse::off_t, fuse::fuse_fill_dir_flags) -> Result<(), ()>,
        offset: fuse::off_t,
        fi: &mut fuse::fuse_file_info,
        flags: fuse::fuse_readdir_flags) -> Result<(), Neg>
    {
        if path != "/" {
            return Err(neg!(-libc::ENOENT));
        }

        let res = filler(".", None, 0, 0)
            .and(filler("..", None, 0, 0))
            .and(filler(&self.filename, None, 0, 0));

        if let Ok(_) = res {
            Ok(())
        } else {
            Err(neg!(-libc::ENOMEM))
        }
    }

    fn open(&mut self,
        path: &str,
        fi: &mut fuse::fuse_file_info) -> Result<(), Neg>
    {
        if &path[1..] != self.filename {
            return Err(neg!(-libc::ENOENT));
        }

        if (fi.flags & libc::O_ACCMODE) != libc::O_RDONLY {
            return Err(neg!(-libc::EACCES));
        }

        Ok(())
    }

    fn read(&mut self,
        path: &str,
        filler: &mut dyn FnMut(&[u8]) -> Result<usize, ()>,
        size: usize,
        offset: fuse::off_t,
        fi: Option<&mut fuse::fuse_file_info>) -> Result<(), Neg>
    {
        if &path[1..] != self.filename {
            return Err(neg!(-libc::ENOENT));
        }

        let mut size = size;
        let offset = offset as usize;

        let len = self.contents.len();

        if offset < len {
            if offset + size > len {
                size = len - offset;
            }

            unwrap!(filler(self.contents[offset .. offset + size].as_bytes()));
        }

        Ok(())
    }
}
