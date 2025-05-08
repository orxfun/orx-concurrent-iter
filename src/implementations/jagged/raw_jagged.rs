use super::{raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};
use alloc::vec::Vec;

pub struct RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    slices: Vec<RawSlice<T>>,
    len: usize,
    indexer: X,
}

impl<'a, T, X> RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    fn from<I>(iter: I, indexer: X) -> Self
    where
        I: Iterator<Item = &'a [T]>,
        T: 'a,
    {
        let mut len = 0;
        let mut slices = match iter.size_hint() {
            (lb, Some(ub)) if lb == ub => Vec::with_capacity(lb),
            _ => Vec::new(),
        };
        for slice in iter {
            len += slice.len();
            slices.push(slice.into())
        }
        Self {
            slices,
            len,
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
        self.slices.iter()
    }
}

impl<T, X> IntoIterator for RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    type Item = RawSlice<T>;

    type IntoIter = alloc::vec::IntoIter<RawSlice<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.into_iter()
    }
}

impl<T, X> RawJagged<T, X>
where
    X: Fn(usize) -> [usize; 2],
{
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn num_slices(&self) -> usize {
        self.slices.len()
    }

    pub fn slice(&self, start: usize, end: usize) -> RawJaggedSlice<T> {
        let begin = (self.indexer)(start);
        let end = (self.indexer)(end);
        RawJaggedSlice::new(&self.slices, begin, end)
    }
}
