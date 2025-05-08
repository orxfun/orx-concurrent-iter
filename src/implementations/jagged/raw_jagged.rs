use super::{raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};
use alloc::vec::Vec;

pub struct RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    jagged: Vec<RawSlice<T>>,
    indexer: X,
}

impl<'a, T, X> RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    fn from<I>(slices: I, indexer: X) -> Self
    where
        I: Iterator<Item = &'a [T]>,
        T: 'a,
    {
        Self {
            jagged: slices.map(Into::into).collect(),
            indexer,
        }
    }
}

impl<'a, T, X> IntoIterator for &'a RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    type Item = &'a RawSlice<T>;

    type IntoIter = core::slice::Iter<'a, RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.jagged.iter()
    }
}

impl<T, X> IntoIterator for RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    type Item = RawSlice<T>;

    type IntoIter = alloc::vec::IntoIter<RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.jagged.into_iter()
    }
}

impl<T, X> RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    pub fn slice(&self, start: usize, end: usize) -> RawJaggedSlice<T> {
        let begin = (self.indexer)(start);
        let end = (self.indexer)(end);
        RawJaggedSlice::new(&self.jagged, begin, end)
    }
}
