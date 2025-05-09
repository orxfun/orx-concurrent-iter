use super::{jagged_index::JaggedIndex, raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};
use alloc::vec::Vec;
use core::cmp::Ordering;

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
    pub fn new<I>(iter: I, indexer: X) -> Self
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

        // slices is never empty, has at least one slice
        let slices = match slices.is_empty() {
            true => [RawSlice::default()].into_iter().collect(),
            false => slices,
        };

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

    pub fn slice(&self, begin: usize, end: usize) -> RawJaggedSlice<T> {
        let begin = self.jagged_index(begin).expect("index-out-of-bounds");
        let end = self.jagged_index(end).expect("index-out-of-bounds");
        RawJaggedSlice::new(&self.slices, begin, end)
    }

    pub fn jagged_index(&self, flat_index: usize) -> Option<JaggedIndex> {
        match flat_index.cmp(&self.len) {
            Ordering::Equal => {
                let f = self.slices.len() - 1;
                let i = self.slices[f].len();
                Some(JaggedIndex::new(f, i))
            }
            Ordering::Less => {
                let [f, i] = (self.indexer)(flat_index);
                Some(JaggedIndex::new(f, i))
            }
            Ordering::Greater => None,
        }
    }
}
