use core::cell::UnsafeCell;

pub struct NumTaken(UnsafeCell<usize>);

impl From<usize> for NumTaken {
    fn from(value: usize) -> Self {
        Self(value.into())
    }
}
