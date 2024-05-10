use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        buffered::{array::BufferedArray, buffered_iter::BufferedIter},
    },
    next::NextChunk,
    AtomicCounter, ConcurrentIter, ExactSizeConcurrentIter, Next,
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
    pub(crate) unsafe fn mut_array(&self) -> &mut [T; N] {
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

    #[inline(always)]
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter().fetch_and_add(number_to_fetch);
        match begin_idx.cmp(&self.initial_len()) {
            Ordering::Less => Some(begin_idx),
            _ => None,
        }
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
    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(n)
            .unwrap_or(self.initial_len());
        let array = unsafe { self.mut_array() };

        let end_idx = (begin_idx + n).min(N).max(begin_idx);

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let idx_range = begin_idx..end_idx;
                let values = idx_range.map(|i| unsafe { get_unchecked(array, i) });
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn early_exit(&self) {
        self.counter().store(N)
    }
}

impl<const N: usize, T: Send + Sync + Default> AtomicIterWithInitialLen<T>
    for ConIterOfArray<N, T>
{
    #[inline(always)]
    fn initial_len(&self) -> usize {
        N
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

    type BufferedIter = BufferedArray<N>;

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

    fn skip_to_end(&self) {
        self.early_exit()
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
}
