use super::as_slice::AsSlice;
use std::marker::PhantomData;

/// Raw representation of a slice defined by a pointer and length.
///
/// All elements in the slice are assumed to be initialized.
pub struct RawSlice<'a, T> {
    ptr: *const T,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T> From<&'a [T]> for RawSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
            phantom: PhantomData,
        }
    }
}

impl<T> RawSlice<'_, T> {
    pub(super) fn new(ptr: *const T, len: usize) -> Self {
        Self {
            ptr,
            len,
            phantom: PhantomData,
        }
    }
}

impl<T> AsSlice<T> for RawSlice<'_, T> {
    fn ptr(&self) -> *const T {
        self.ptr
    }

    fn length(&self) -> usize {
        self.len
    }

    fn raw_slice(&self, begin: usize, len: usize) -> RawSlice<T> {
        debug_assert!(begin + len <= self.len);
        let ptr = unsafe { self.ptr.add(begin) };
        Self::new(ptr, len)
    }
}
