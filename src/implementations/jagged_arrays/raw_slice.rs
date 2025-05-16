use super::as_slice::AsSlice;

pub struct RawSlice<T> {
    ptr: *const T,
    len: usize,
}

impl<T> RawSlice<T> {
    pub fn new(ptr: *const T, len: usize) -> Self {
        Self { ptr, len }
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
