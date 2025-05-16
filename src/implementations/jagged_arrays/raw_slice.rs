use super::as_slice::AsSlice;

/// Raw representation of a slice defined by a pointer and length.
///
/// All elements in the slice are assumed to be initialized.
pub struct RawSlice<T> {
    ptr: *const T,
    len: usize,
}

impl<'a, T> From<&'a [T]> for RawSlice<T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl<T> RawSlice<T> {
    pub(super) fn new(ptr: *const T, len: usize) -> Self {
        Self { ptr, len }
    }

    /// # SAFETY
    ///
    /// The caller must ensure that the slice does not overlive the data source jagged array.
    pub(super) unsafe fn as_slice<'b>(self) -> &'b [T] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// # SAFETY
    ///
    /// The caller must ensure that the reference does not outlive the data source jagged array.
    pub(super) unsafe fn get_unchecked<'b>(&self, i: usize) -> &'b T {
        debug_assert!(i < self.len);
        unsafe { &*self.ptr.add(i) }
    }
}

impl<T> AsSlice<T> for RawSlice<T> {
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
