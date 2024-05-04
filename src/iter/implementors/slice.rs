use super::cloned::slice::ClonedConIterOfSlice;
use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        buffered::{buffered_iter::BufferedIter, slice::BufferedSlice},
    },
    next::NextChunk,
    ConcurrentIter, ExactSizeConcurrentIter, Next,
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

    /// A concurrent iterator over a slice yielding references to clones of the elements.
    pub fn cloned(self) -> ClonedConIterOfSlice<'a, T>
    where
        T: Clone,
    {
        self.into()
    }

    /// Returns a reference to the underlying slice.
    pub(crate) fn as_slice(&self) -> &'a [T] {
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

impl<'a, T: Send + Sync> AtomicIter<&'a T> for ConIterOfSlice<'a, T> {
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    #[inline(always)]
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter().fetch_and_add(number_to_fetch);
        match begin_idx.cmp(&self.initial_len()) {
            Ordering::Less => Some(begin_idx),
            _ => None,
        }
    }

    #[inline(always)]
    fn get(&self, item_idx: usize) -> Option<&'a T> {
        self.slice.get(item_idx)
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> Option<NextChunk<&'a T, impl ExactSizeIterator<Item = &'a T>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(n)
            .unwrap_or(self.initial_len());
        let end_idx = (begin_idx + n).min(self.initial_len()).max(begin_idx);

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let values = self.slice[begin_idx..end_idx].iter();
                Some(NextChunk { begin_idx, values })
            }
        }
    }
}

impl<'a, T: Send + Sync> AtomicIterWithInitialLen<&'a T> for ConIterOfSlice<'a, T> {
    #[inline(always)]
    fn initial_len(&self) -> usize {
        self.slice.len()
    }
}

unsafe impl<'a, T: Send + Sync> Sync for ConIterOfSlice<'a, T> {}

unsafe impl<'a, T: Send + Sync> Send for ConIterOfSlice<'a, T> {}

// AtomicIter -> ConcurrentIter

impl<'a, T: Send + Sync> ConcurrentIter for ConIterOfSlice<'a, T> {
    type Item = &'a T;

    type BufferedIter = BufferedSlice;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        self.fetch_n(chunk_size)
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        Some(<Self as ExactSizeConcurrentIter>::len(self))
    }
}

impl<'a, T: Send + Sync> ExactSizeConcurrentIter for ConIterOfSlice<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        let current = <Self as AtomicIter<_>>::counter(self).current();
        let initial_len = <Self as AtomicIterWithInitialLen<_>>::initial_len(self);
        match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        }
    }
}
