use super::as_raw_slice::AsRawSlice;

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
}

impl<T> AsRawSlice<T> for RawSlice<T> {
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
