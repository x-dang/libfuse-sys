#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


use std::ffi::CString;
use std::os::raw::{ c_int, c_char, c_void };


pub mod operations;


pub fn fuse_main<T, U>(args: T, ops: U) -> i32
    where T: Iterator<Item=String>,
          U: 'static + operations::FuseOperations
{
    let c_args = args
        .map(|arg| CString::new(arg.clone()).unwrap());

    let mut c_args: Vec<_> = c_args
        .map(|arg| arg.into_raw())
        .collect();

    operations::set_operations(ops);

    let ops = fuse_operations::new();

    unsafe {
        let ret = fuse_main_real(
            c_args.len() as c_int,
            c_args.as_mut_ptr(),
            &ops,
            std::mem::size_of::<fuse_operations>(),
            std::ptr::null_mut());

        let _: Vec<_> = c_args
            .iter()
            .map(|raw| CString::from_raw(*raw))
            .collect();

        ret
    }
}
