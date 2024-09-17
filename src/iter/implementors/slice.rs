use crate::{
    iter::buffered::slice::BufferedSlice, next::NextChunk, ConcurrentIter, ConcurrentIterX, Next,
};
use core::sync::atomic::{AtomicUsize, Ordering};

/// A concurrent iterator over a slice yielding references to the elements.
pub struct ConIterOfSlice<'a, T: Send + Sync> {
    slice: &'a [T],
    counter: AtomicUsize,
}

impl<'a, T: Send + Sync> core::fmt::Debug for ConIterOfSlice<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        super::helpers::fmt_iter(f, "ConIterOfSlice", Some(self.slice.len()), &self.counter)
    }
}

impl<'a, T: Send + Sync> ConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: 0.into(),
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
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.slice.len() {
            true => Some(begin_idx),
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
        let counter = self.counter.load(Ordering::SeqCst).into();
        Self {
            slice: self.slice,
            counter,
        }
    }
}

unsafe impl<'a, T: Send + Sync> Sync for ConIterOfSlice<'a, T> {}

unsafe impl<'a, T: Send + Sync> Send for ConIterOfSlice<'a, T> {}

// AtomicIter -> ConcurrentIter

impl<'a, T: Send + Sync> ConcurrentIterX for ConIterOfSlice<'a, T> {
    type Item = &'a T;

    type SeqIter = core::iter::Skip<core::slice::Iter<'a, T>>;

    type BufferedIterX = BufferedSlice<T>;

    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
    }

    fn next_chunk_x(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = Self::Item>> {
        let begin_idx = self
            .progress_and_get_begin_idx(chunk_size)
            .unwrap_or(self.slice.len());
        let end_idx = (begin_idx + chunk_size)
            .min(self.slice.len())
            .max(begin_idx);

        match begin_idx < end_idx {
            true => Some(self.slice[begin_idx..end_idx].iter()),
            false => None,
        }
    }

    fn next(&self) -> Option<Self::Item> {
        let idx = self.counter.fetch_add(1, Ordering::Acquire);
        self.get(idx)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        let current = self.counter.load(Ordering::Acquire);
        let initial_len = self.slice.len();
        let len = match current.cmp(&initial_len) {
            core::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        };
        Some(len)
    }

    #[inline(always)]
    fn try_get_initial_len(&self) -> Option<usize> {
        Some(self.slice.len())
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }
}

impl<'a, T: Send + Sync> ConcurrentIter for ConIterOfSlice<'a, T> {
    type BufferedIter = Self::BufferedIterX;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        let idx = self.counter.fetch_add(1, Ordering::Acquire);
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

        match begin_idx < end_idx {
            true => {
                let values = self.slice[begin_idx..end_idx].iter();
                Some(NextChunk { begin_idx, values })
            }
            false => None,
        }
    }
}
