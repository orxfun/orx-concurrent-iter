use crate::implementations::jagged_arrays::{
    as_raw_slice::{AsOwningSlice, AsRawSlice},
    raw_slice::RawSlice,
};
use alloc::vec::Vec;
use std::mem::ManuallyDrop;

/// Raw representation of a vector defined by a pointer, capacity and length.
///
/// All elements within the length of the vector are assumed to be initialized;
/// elements between length and capacity are assumed to be uninitialized.
///
/// # SAFETY
///
/// Does not release memory on Drop.
pub struct RawVec<T> {
    ptr: *const T,
    len: usize,
    capacity: usize,
}

impl<T> From<Vec<T>> for RawVec<T> {
    fn from(value: Vec<T>) -> Self {
        let raw = Self {
            ptr: value.as_ptr(),
            len: value.len(),
            capacity: value.capacity(),
        };
        let _ = ManuallyDrop::new(value);
        raw
    }
}

impl<T> AsRawSlice<T> for RawVec<T> {
    fn ptr(&self) -> *const T {
        self.ptr
    }

    fn length(&self) -> usize {
        self.len
    }

    fn raw_slice(&self, begin: usize, len: usize) -> RawSlice<T> {
        debug_assert!(begin + len <= self.len);
        let ptr = unsafe { self.ptr.add(begin) };
        RawSlice::new(ptr, len)
    }
}

impl<T> AsOwningSlice<T> for RawVec<T> {
    fn capacity(&self) -> usize {
        self.capacity
    }
}
