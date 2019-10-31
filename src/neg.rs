use libc::c_int;


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
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

#[macro_export]
macro_rules! neg {
    ( $v:expr ) => {
        {
            let v = $v;

            if v < 0 {
                Neg::new(v).unwrap()
            } else {
                panic!("integer used to construct `Neg` must be less than zero, found `{}`", v)
            }
        }
    };
}
