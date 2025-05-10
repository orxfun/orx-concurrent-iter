use super::raw_slice::RawSlice;
use alloc::vec::Vec;
use std::mem::ManuallyDrop;

pub struct RawVec<T> {
    ptr: *const T,
    len: usize,
    capacity: usize,
}

impl<T> Default for RawVec<T> {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null(),
            len: 0,
            capacity: 0,
        }
    }
}

impl<T> Clone for RawVec<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            capacity: self.capacity,
        }
    }
}

impl<T> From<Vec<T>> for RawVec<T> {
    fn from(value: Vec<T>) -> Self {
        let value = ManuallyDrop::new(value);
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
            capacity: value.capacity(),
        }
    }
}

impl<T> From<&[T]> for RawVec<T> {
    fn from(value: &[T]) -> Self {
        let value = ManuallyDrop::new(value);
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
            capacity: value.len(),
        }
    }
}

impl<T> RawVec<T> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_raw_slice(&self) -> RawSlice<T> {
        RawSlice::new(self.ptr, self.len)
    }

    pub fn raw_slice(&self, begin: usize, end: usize) -> RawSlice<T> {
        let ptr = unsafe { self.ptr.add(begin) };
        let len = end - begin;
        RawSlice::new(ptr, len)
    }

    /// Returns the slice from the raw slice for elements in range [start..start+len].
    ///
    /// Returns None if the range is empty or out of bounds.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub fn slice(&self, start: usize, len: usize) -> Option<&[T]> {
        match start + len <= self.len && len > 0 {
            true => {
                let ptr = unsafe { self.ptr.add(start) }; // ptr + start is in bounds
                Some(unsafe { core::slice::from_raw_parts(ptr, len) }) // ptr + start + len is in bounds
            }
            false => None,
        }
    }

    pub unsafe fn drop_elements_in_place(&self, begin: usize) {
        for i in begin..self.len {
            let ptr = unsafe { self.ptr.add(i) as *mut T };
            unsafe { ptr.drop_in_place() };
        }
    }

    pub unsafe fn drop_allocation(&self) {
        let _vec_to_drop = unsafe { Vec::from_raw_parts(self.ptr as *mut T, 0, self.capacity) };
    }

    /// Returns pointers to the first and last, (len-1)-th, element of the vector.
    ///
    /// If the vector is empty, both pointers are null.
    pub fn first_and_last_ptrs(&self) -> [*const T; 2] {
        match self.len {
            0 => [core::ptr::null(), core::ptr::null()],
            n => {
                let last = unsafe { self.ptr.add(n - 1) };
                [self.ptr, last]
            }
        }
    }

    pub unsafe fn ptr_at(&self, position: usize) -> *const T {
        unsafe { self.ptr.add(position) }
    }
}
