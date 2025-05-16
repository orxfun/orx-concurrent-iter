use alloc::vec::Vec;

pub trait AsSlice<T> {
    fn as_slice(&self) -> &[T];
}

impl<T> AsSlice<T> for &[T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T> AsSlice<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }
}
