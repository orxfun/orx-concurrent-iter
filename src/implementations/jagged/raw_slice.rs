pub struct RawSlice<T> {
    ptr: *const T,
    len: usize,
}

impl<T> From<&[T]> for RawSlice<T> {
    fn from(slice: &[T]) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
        }
    }
}

impl<T> RawSlice<T> {
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the slice from the raw slice for elements in range [start..start+len].
    ///
    /// Returns None if the range is empty or out of bounds.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    fn slice(&self, start: usize, len: usize) -> Option<&[T]> {
        match start + len <= self.len && len > 0 {
            true => {
                let ptr = unsafe { self.ptr.add(start) }; // ptr + start is in bounds
                Some(unsafe { core::slice::from_raw_parts(ptr, len) }) // ptr + start + len is in bounds
            }
            false => None,
        }
    }

    /// Returns the slice from the raw slice for elements in range [start..].
    ///
    /// Returns None if the range is empty or out of bounds.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub fn slice_from(&self, start: usize) -> Option<&[T]> {
        let len = self.len.saturating_sub(start);
        self.slice(start, len)
    }
}
