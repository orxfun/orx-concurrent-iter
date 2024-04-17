use crate::{
    iter::atomic_iter::AtomicIterWithInitialLen, AtomicCounter, AtomicIter, NextChunk,
    NextManyExact,
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

impl<const N: usize, T: Send + Sync + Default> AtomicIter for ConIterOfArray<N, T> {
    type Item = T;

    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    fn get(&self, item_idx: usize) -> Option<Self::Item> {
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

    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n_with_exact_len(n)
    }
}

impl<const N: usize, T: Send + Sync + Default> AtomicIterWithInitialLen for ConIterOfArray<N, T> {
    fn initial_len(&self) -> usize {
        N
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>> {
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
