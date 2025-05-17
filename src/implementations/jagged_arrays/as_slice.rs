use super::AsRawSlice;
use alloc::vec::Vec;

/// A type that can be represented as a contagious slice.
pub trait AsSlice<T>: AsRawSlice<T> {
    /// Returns the slice.
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
