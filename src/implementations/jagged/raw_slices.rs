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

impl<'a, T> IntoIterator for &'a RawSlices<T> {
    type Item = &'a RawSlice<T>;

    type IntoIter = core::slice::Iter<'a, RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.iter()
    }
}

impl<T> IntoIterator for RawSlices<T> {
    type Item = RawSlice<T>;

    type IntoIter = alloc::vec::IntoIter<RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.into_iter()
    }
}
