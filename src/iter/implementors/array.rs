use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        default_fns,
    },
    AtomicCounter, ConcurrentIter, ExactSizeConcurrentIter, Next, NextChunk, NextManyExact,
};
use std::{cell::UnsafeCell, cmp::Ordering};

/// A concurrent iterator over an array, consuming the array and yielding its elements.
#[derive(Debug)]
pub struct ConIterOfArray<const N: usize, T: Send + Sync + Default> {
    array: UnsafeCell<[T; N]>,
    counter: AtomicCounter,
}

impl<const N: usize, T: Send + Sync + Default> ConIterOfArray<N, T> {
    /// Consumes and creates a concurrent iterator of the given `array`.
    pub fn new(array: [T; N]) -> Self {
        Self {
            array: array.into(),
            counter: AtomicCounter::new(),
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    unsafe fn mut_array(&self) -> &mut [T; N] {
        unsafe { &mut *self.array.get() }
    }
}

impl<const N: usize, T: Send + Sync + Default> From<[T; N]> for ConIterOfArray<N, T> {
    /// Consumes and creates a concurrent iterator of the given `array`.
    fn from(array: [T; N]) -> Self {
        Self::new(array)
    }
}

impl<const N: usize, T: Send + Sync + Default> AtomicIter<T> for ConIterOfArray<N, T> {
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    fn get(&self, item_idx: usize) -> Option<T> {
        match item_idx.cmp(&N) {
            Ordering::Less => {
                // SAFETY: only one thread can access the `item_idx`-th position and `item_idx` is in bounds
                let array = unsafe { self.mut_array() };
                let value = std::mem::take(&mut array[item_idx]);
                Some(value)
            }
            _ => None,
        }
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

impl<const N: usize, T: Send + Sync + Default> AtomicIterWithInitialLen<T>
    for ConIterOfArray<N, T>
{
    #[inline(always)]
    fn initial_len(&self) -> usize {
        N
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<T, impl ExactSizeIterator<Item = T>> {
        let begin_idx = self.counter().fetch_and_add(n);
        let array = unsafe { self.mut_array() };

        let end_idx = (begin_idx + n).min(N).max(begin_idx);
        let idx_range = begin_idx..end_idx;
        let values = idx_range.map(|i| unsafe { get_unchecked(array, i) });

        NextManyExact { begin_idx, values }
    }
}

#[inline(always)]
unsafe fn get_unchecked<const N: usize, T: Default>(array: &mut [T; N], item_idx: usize) -> T {
    std::mem::take(&mut array[item_idx])
}

unsafe impl<const N: usize, T: Send + Sync + Default> Sync for ConIterOfArray<N, T> {}

unsafe impl<const N: usize, T: Send + Sync + Default> Send for ConIterOfArray<N, T> {}

// AtomicIter -> ConcurrentIter

impl<const N: usize, T: Send + Sync + Default> ConcurrentIter for ConIterOfArray<N, T> {
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

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        Some(<Self as ExactSizeConcurrentIter>::len(self))
    }
}

impl<const N: usize, T: Send + Sync + Default> ExactSizeConcurrentIter for ConIterOfArray<N, T> {
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
