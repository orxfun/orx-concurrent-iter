use super::raw_slice::RawSlice;
use alloc::vec::Vec;

pub struct RawSlices<'a, T> {
    slices: Vec<RawSlice<'a, T>>,
}

impl<'a, T, I> From<I> for RawSlices<'a, T>
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
