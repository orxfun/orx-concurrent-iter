use super::as_slice::AsSlice;

pub struct RawSlice<T> {
    ptr: *const T,
    len: usize,
}

impl<T> AsSlice<T> for RawSlice<T> {
    fn ptr(&self) -> *const T {
        self.ptr
    }

    fn length(&self) -> usize {
        self.len
    }

    fn slice(&self, begin: usize, len: usize) -> &[T] {
        debug_assert!(begin + len <= self.len);
        let data = unsafe { self.ptr.add(begin) };
        unsafe { core::slice::from_raw_parts(data, len) }
    }
}
