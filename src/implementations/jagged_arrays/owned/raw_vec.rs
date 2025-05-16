use crate::implementations::jagged_arrays::{
    as_slice::{AsOwningSlice, AsSlice},
    raw_slice::RawSlice,
};
use alloc::vec::Vec;
use std::mem::ManuallyDrop;

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

impl<T> AsSlice<T> for RawVec<T> {
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
