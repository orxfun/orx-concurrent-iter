use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        default_fns,
    },
    ConIterOfSlice, ConcurrentIter, ExactSizeConcurrentIter, Next, NextChunk, NextManyExact,
};
use std::cmp::Ordering;

/// A concurrent iterator over a slice yielding references to clones of the elements.
pub struct ClonedConIterOfSlice<'a, T: Send + Sync + Clone> {
    slice: &'a [T],
    counter: AtomicCounter,
}

impl<'a, T: Send + Sync + Clone> ClonedConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: AtomicCounter::new(),
        }
    }
}

impl<'a, T: Send + Sync + Clone> From<&'a [T]> for ClonedConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    fn from(slice: &'a [T]) -> Self {
        Self::new(slice)
    }
}

impl<'a, T: Send + Sync + Clone> From<ConIterOfSlice<'a, T>> for ClonedConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    fn from(iter: ConIterOfSlice<'a, T>) -> Self {
        Self {
            slice: iter.as_slice(),
            counter: iter.counter().clone(),
        }
    }
}

impl<'a, T: Send + Sync + Clone> Clone for ClonedConIterOfSlice<'a, T> {
    fn clone(&self) -> Self {
        Self {
            slice: self.slice,
            counter: self.counter.clone(),
        }
    }
}

impl<'a, T: Send + Sync + Clone> AtomicIter<T> for ClonedConIterOfSlice<'a, T> {
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    #[inline(always)]
    fn get(&self, item_idx: usize) -> Option<T> {
        self.slice.get(item_idx).cloned()
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> impl NextChunk<T> {
        self.fetch_n_with_exact_len(n)
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(T)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::exact_for_each(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, T)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::exact_for_each_with_ids(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, F: FnMut(B, T) -> B>(&self, chunk_size: usize, initial: B, f: F) -> B {
        default_fns::fold::exact_fold(self, chunk_size, f, initial)
    }
}

impl<'a, T: Send + Sync + Clone> AtomicIterWithInitialLen<T> for ClonedConIterOfSlice<'a, T> {
    #[inline(always)]
    fn initial_len(&self) -> usize {
        self.slice.len()
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<T, impl ExactSizeIterator<Item = T>> {
        let begin_idx = self.counter().fetch_and_add(n);

        let values = match begin_idx.cmp(&self.slice.len()) {
            Ordering::Less => {
                let end_idx = (begin_idx + n).min(self.slice.len()).max(begin_idx);
                let values = self.slice[begin_idx..end_idx].iter().cloned();
                values
            }
            _ => [].iter().cloned(),
        };

        NextManyExact { begin_idx, values }
    }
}

unsafe impl<'a, T: Send + Sync + Clone> Sync for ClonedConIterOfSlice<'a, T> {}

unsafe impl<'a, T: Send + Sync + Clone> Send for ClonedConIterOfSlice<'a, T> {}

// AtomicIter -> ConcurrentIter

impl<'a, T: Send + Sync + Clone> ConcurrentIter for ClonedConIterOfSlice<'a, T> {
    type Item = T;

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

impl<'a, T: Send + Sync + Clone> ExactSizeConcurrentIter for ClonedConIterOfSlice<'a, T> {
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
