use core::marker::PhantomData;

pub struct RawSlice<'a, T> {
    ptr: *const T,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T> From<&'a [T]> for RawSlice<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
            phantom: PhantomData,
        }
    }
}
