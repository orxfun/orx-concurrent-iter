use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        buffered::{
            buffered_chunk::BufferedChunk, buffered_iter::BufferedIter, slice::BufferedSlice,
        },
    },
    next::NextChunk,
    ConcurrentIter, Next,
};
use std::cmp::Ordering;

/// A concurrent iterator over a slice yielding references to the elements.
pub struct ConIterOfSlice<'a, T: Send + Sync> {
    slice: &'a [T],
    counter: AtomicCounter,
}

impl<'a, T: Send + Sync> std::fmt::Debug for ConIterOfSlice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::helpers::fmt_iter(f, "ConIterOfSlice", Some(self.slice.len()), &self.counter)
    }
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
    pub(crate) fn as_slice(&self) -> &'a [T] {
        self.slice
    }

    #[inline(always)]
    fn get(&self, item_idx: usize) -> Option<&'a T> {
        self.slice.get(item_idx)
    }

    #[inline(always)]
    pub(crate) fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_and_add(number_to_fetch);
        match begin_idx.cmp(&self.slice.len()) {
            Ordering::Less => Some(begin_idx),
            _ => None,
        }
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

unsafe impl<'a, T: Send + Sync> Sync for ConIterOfSlice<'a, T> {}

unsafe impl<'a, T: Send + Sync> Send for ConIterOfSlice<'a, T> {}

// AtomicIter -> ConcurrentIter

impl<'a, T: Send + Sync> ConcurrentIter for ConIterOfSlice<'a, T> {
    type Item = &'a T;

    type BufferedIter = BufferedSlice<T>;

    type SeqIter = std::iter::Skip<std::slice::Iter<'a, T>>;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    /// let slice = vec.as_slice();
    /// let con_iter = slice.into_con_iter();
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
    ///     assert_eq!(x, &(num_used + i).to_string());
    /// }
    /// ```
    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.current();
        self.slice.iter().skip(current)
    }

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        let idx = self.counter.fetch_and_increment();
        self.get(idx).map(|value| Next { idx, value })
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(chunk_size)
            .unwrap_or(self.slice.len());
        let end_idx = (begin_idx + chunk_size)
            .min(self.slice.len())
            .max(begin_idx);

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let values = self.slice[begin_idx..end_idx].iter();
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        let current = self.counter.current();
        let initial_len = self.slice.len();
        let len = match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        };
        Some(len)
    }

    fn skip_to_end(&self) {
        let _ = self.counter.get_current_max_value(self.slice.len());
    }
}
