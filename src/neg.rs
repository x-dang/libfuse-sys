use std::clone::Clone;
use libc::c_int;


#[derive(Copy)]
pub struct Neg(c_int);

impl Neg {
    pub fn new(v: c_int) -> Option<Self> {
        if v < 0 {
            Some(Self(v))
        } else {
            None
        }
    }

    pub const fn get(self) -> c_int {
        self.0
    }
}

impl Clone for Neg {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
