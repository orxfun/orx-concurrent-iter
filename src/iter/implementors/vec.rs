use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        buffered::{buffered_chunk::BufferedChunk, buffered_iter::BufferedIter, vec::BufferedVec},
    },
    next::NextChunk,
    AtomicCounter, ConcurrentIter, Next,
};
use std::{cell::UnsafeCell, cmp::Ordering};

/// A concurrent iterator over a vector, consuming the vector and yielding its elements.
#[derive(Debug)]
pub struct ConIterOfVec<T: Send + Sync + Default> {
    vec: UnsafeCell<Vec<T>>,
    vec_len: usize,
    counter: AtomicCounter,
}

impl<T: Send + Sync + Default> ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            vec_len: vec.len(),
            vec: vec.into(),
            counter: AtomicCounter::new(),
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn mut_vec(&self) -> &mut Vec<T> {
        unsafe { &mut *self.vec.get() }
    }
}

impl<T: Send + Sync + Default> From<Vec<T>> for ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T: Send + Sync + Default> AtomicIter<T> for ConIterOfVec<T> {
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
        match item_idx.cmp(&self.vec_len) {
            Ordering::Less => {
                // SAFETY: only one thread can access the `item_idx`-th position and `item_idx` is in bounds
                let vec = unsafe { self.mut_vec() };
                let value = std::mem::take(&mut vec[item_idx]);
                Some(value)
            }
            _ => None,
        }
    }

    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        let vec = unsafe { self.mut_vec() };

        let begin_idx = self
            .progress_and_get_begin_idx(n)
            .unwrap_or(self.initial_len());
        let end_idx = (begin_idx + n).min(self.initial_len()).max(begin_idx);

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let idx_range = begin_idx..end_idx;
                let values = idx_range.map(|i| unsafe { get_unchecked(vec, i) });
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn early_exit(&self) {
        self.counter().store(self.vec_len)
    }
}

impl<T: Send + Sync + Default> AtomicIterWithInitialLen<T> for ConIterOfVec<T> {
    #[inline(always)]
    fn initial_len(&self) -> usize {
        self.vec_len
    }
}

#[inline(always)]
unsafe fn get_unchecked<T: Default>(vec: &mut [T], item_idx: usize) -> T {
    std::mem::take(&mut vec[item_idx])
}

unsafe impl<T: Send + Sync + Default> Sync for ConIterOfVec<T> {}

unsafe impl<T: Send + Sync + Default> Send for ConIterOfVec<T> {}

// AtomicIter -> ConcurrentIter

impl<T: Send + Sync + Default> ConcurrentIter for ConIterOfVec<T> {
    type Item = T;

    type BufferedIter = BufferedVec<T>;

    type SeqIter = std::iter::Skip<std::vec::IntoIter<T>>;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    /// let con_iter = vec.into_con_iter();
    ///
    /// std::thread::scope(|s| {
    ///     s.spawn(|| {
    ///         for _ in 0..42 {
    ///             _ = con_iter.next();
    ///         }
    ///
    ///         let mut buffered = con_iter.buffered_iter(32);
    ///         let _chunk = buffered.next().unwrap();
    ///     });
    /// });
    ///
    /// let num_used = 42 + 32;
    ///
    /// // converts the remaining elements into a sequential iterator
    /// let seq_iter = con_iter.into_seq_iter();
    ///
    /// assert_eq!(seq_iter.len(), 1024 - num_used);
    /// for (i, x) in seq_iter.enumerate() {
    ///     assert_eq!(x, (num_used + i).to_string());
    /// }
    /// ```
    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter().current();
        let vec = self.vec.into_inner();
        vec.into_iter().skip(current)
    }

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
        let current = <Self as AtomicIter<_>>::counter(self).current();
        let initial_len = <Self as AtomicIterWithInitialLen<_>>::initial_len(self);
        let len = match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        };
        Some(len)
    }

    fn skip_to_end(&self) {
        self.early_exit()
    }
}
