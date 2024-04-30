use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        default_fns,
    },
    ConcurrentIter, ExactSizeConcurrentIter, Next, NextChunk, NextManyExact,
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

impl<'a, T: Send + Sync> AtomicIter<&'a T> for ConIterOfSlice<'a, T> {
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    #[inline(always)]
    fn get(&self, item_idx: usize) -> Option<&'a T> {
        self.slice.get(item_idx)
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> impl NextChunk<&'a T> {
        self.fetch_n_with_exact_len(n)
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(&'a T)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::exact_for_each(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, &'a T)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::exact_for_each_with_ids(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, F: FnMut(B, &'a T) -> B>(&self, chunk_size: usize, initial: B, f: F) -> B {
        default_fns::fold::exact_fold(self, chunk_size, f, initial)
    }
}

impl<'a, T: Send + Sync> AtomicIterWithInitialLen<&'a T> for ConIterOfSlice<'a, T> {
    #[inline(always)]
    fn initial_len(&self) -> usize {
        self.slice.len()
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<&'a T, impl ExactSizeIterator<Item = &'a T>> {
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

// AtomicIter -> ConcurrentIter

impl<'a, T: Send + Sync> ConcurrentIter for ConIterOfSlice<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(&self, chunk_size: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n(chunk_size)
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(Self::Item)>(&self, chunk_size: usize, f: F) {
        <Self as AtomicIter<_>>::for_each_n(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, Self::Item)>(&self, chunk_size: usize, f: F) {
        <Self as AtomicIter<_>>::enumerate_for_each_n(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, Fold>(&self, chunk_size: usize, neutral: B, fold: Fold) -> B
    where
        Fold: FnMut(B, Self::Item) -> B,
    {
        <Self as AtomicIter<_>>::fold(self, chunk_size, neutral, fold)
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

    fn next_exact_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        let next = <Self as AtomicIterWithInitialLen<_>>::fetch_n_with_exact_len(self, chunk_size);
        if next.is_empty() {
            None
        } else {
            Some(next)
        }
    }
}
