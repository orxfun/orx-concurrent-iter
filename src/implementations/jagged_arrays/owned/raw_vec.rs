use crate::implementations::jagged_arrays::{
    as_raw_slice::{AsOwningSlice, AsRawSlice},
    raw_slice::RawSlice,
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

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

impl<T> RawVec<T> {
    /// Creates the raw vec from `vec`.
    ///
    /// # SAFETY
    ///
    /// `RawVec` created from `vec` does not drop allocation of the vec when it is dropped.
    /// Must call [`AsOwningSlice::drop_allocation`] to prevent memory leaks.
    pub unsafe fn new_from_vec(vec: Vec<T>) -> Self {
        let raw = Self {
            ptr: vec.as_ptr(),
            len: vec.len(),
            capacity: vec.capacity(),
        };
        let _ = ManuallyDrop::new(vec);
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
        assert!(begin.checked_add(len).is_some_and(|end| end <= self.len));
        let ptr = unsafe { self.ptr.add(begin) };
        RawSlice::new(ptr, len)
    }

    unsafe fn raw_slice_unchecked(&self, begin: usize, len: usize) -> RawSlice<T> {
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
