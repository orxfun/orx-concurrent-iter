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

impl<'a, T> IntoIterator for RawSlice<'a, T>
where
    T: 'a,
{
    type Item = &'a T;

    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

impl<'a, T> Clone for RawSlice<'a, T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone(),
            len: self.len.clone(),
            phantom: self.phantom.clone(),
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

    /// # SAFETY
    ///
    /// The caller must ensure that the slice does not overlive the data source jagged array.
    pub(super) unsafe fn as_slice<'b>(self) -> &'b [T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
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
