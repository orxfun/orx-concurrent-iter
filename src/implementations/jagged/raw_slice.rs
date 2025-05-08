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
}
