use super::raw_slice::RawSlice;
use alloc::vec::Vec;

pub struct RawSlices<T> {
    slices: Vec<RawSlice<T>>,
}

impl<'a, T, I> From<I> for RawSlices<T>
where
    T: 'a,
    I: Iterator<Item = &'a [T]>,
{
    fn from(slices: I) -> Self {
        Self {
            slices: slices.map(Into::into).collect(),
        }
    }
}
