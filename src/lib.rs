pub mod fuse;

mod neg;
mod operations;


pub use neg::Neg;
pub use operations::Operations;

use std::ffi::CString;
use std::convert::TryInto;


pub fn fuse_main<T, U>(args: T, ops: U) -> Result<(), i32>
    where T: Iterator<Item=String>,
          U: 'static + Operations
{
    let c_args = args
        .map(|arg| CString::new(arg.clone()).unwrap());

    let mut c_args: Vec<_> = c_args
        .map(|arg| arg.into_raw())
        .collect();

    let ops = operations::set_operations(ops);

    unsafe {
        let err = fuse::fuse_main_real(
            c_args.len().try_into().unwrap(),
            c_args.as_mut_ptr(),
            &ops,
            std::mem::size_of::<fuse::fuse_operations>(),
            std::ptr::null_mut());

        let _: Vec<_> = c_args
            .iter()
            .map(|raw| CString::from_raw(*raw))
            .collect();

        if err == 0 {
            Ok(())
        } else {
            Err(err.try_into().unwrap())
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
