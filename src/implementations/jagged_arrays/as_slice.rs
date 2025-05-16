use super::AsRawSlice;
use alloc::vec::Vec;

pub trait AsSlice<T>: AsRawSlice<T> {
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
