use super::{raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};
use alloc::vec::Vec;

pub struct RawJagged<T, X> {
    slices: Vec<RawSlice<T>>,
    indexer: X,
}

impl<'a, T, X> RawJagged<T, X> {
    fn from<I>(slices: I, indexer: X) -> Self
    where
        I: Iterator<Item = &'a [T]>,
        T: 'a,
    {
        Self {
            slices: slices.map(Into::into).collect(),
            indexer,
        }
    }
}

impl<'a, T, X> IntoIterator for &'a RawJagged<T, X> {
    type Item = &'a RawSlice<T>;

    type IntoIter = core::slice::Iter<'a, RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.iter()
    }
}

impl<T, X> IntoIterator for RawJagged<T, X> {
    type Item = RawSlice<T>;

    type IntoIter = alloc::vec::IntoIter<RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.into_iter()
    }
}

impl<T, X> RawJagged<T, X> {
    pub fn slice(&self, start: usize, end: usize) -> RawJaggedSlice<T, X> {
        todo!()
    }
}
