use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
    },
    NextChunk, NextManyExact,
};
use std::cmp::Ordering;

/// A concurrent iterator over a slice yielding references to the elements.
#[derive(Debug)]
pub struct ConIterOfSlice<'a, T: Send + Sync> {
    slice: &'a [T],
    counter: AtomicCounter,
}

impl<'a, T: Send + Sync> ConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: AtomicCounter::new(),
        }
    }

    /// Returns a reference to the underlying slice.
    pub fn as_slice(&self) -> &'a [T] {
        self.slice
    }
}

impl<'a, T: Send + Sync> From<&'a [T]> for ConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    fn from(slice: &'a [T]) -> Self {
        Self::new(slice)
    }
}

impl<'a, T: Send + Sync> Clone for ConIterOfSlice<'a, T> {
    fn clone(&self) -> Self {
        Self {
            slice: self.slice,
            counter: self.counter.clone(),
        }
    }
}

impl<'a, T: Send + Sync> AtomicIter for ConIterOfSlice<'a, T> {
    type Item = &'a T;

    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    fn get(&self, item_idx: usize) -> Option<Self::Item> {
        self.slice.get(item_idx)
    }

    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n_with_exact_len(n)
    }
}

impl<'a, T: Send + Sync> AtomicIterWithInitialLen for ConIterOfSlice<'a, T> {
    fn initial_len(&self) -> usize {
        self.slice.len()
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>> {
        let begin_idx = self.counter().fetch_and_add(n);

        let values = match begin_idx.cmp(&self.slice.len()) {
            Ordering::Less => {
                let end_idx = (begin_idx + n).min(self.slice.len()).max(begin_idx);
                let values = self.slice[begin_idx..end_idx].iter();
                values
            }
            _ => [].iter(),
        };

        NextManyExact { begin_idx, values }
    }
}

unsafe impl<'a, T: Send + Sync> Sync for ConIterOfSlice<'a, T> {}

unsafe impl<'a, T: Send + Sync> Send for ConIterOfSlice<'a, T> {}
