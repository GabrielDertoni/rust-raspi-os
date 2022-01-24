use core::fmt::{ Debug, Display };

use crate::allocators::KBox;

pub trait Error: Debug + Display {}

pub type KError = KBox<'static, dyn Error + 'static>;

impl Error for &'static str {}

impl From<&'static str> for KBox<'static, dyn Error> {
    fn from(val: &'static str) -> Self {
        // FIXME: This is not Ok. It is a safe interface to an unsafe function.
        unsafe { KBox::new(val) }
    }
}
