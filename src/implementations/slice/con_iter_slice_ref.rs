use super::chunks_iter_slice_ref::ChunksIterSliceRef;
use crate::{concurrent_iter::ConcurrentIter, next::Next};
use core::{
    iter::Skip,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterSliceRef<'a, T> {
    slice: &'a [T],
    counter: AtomicUsize,
}

impl<'a, T> ConIterSliceRef<'a, T> {
    pub(crate) fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: 0.into(),
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.slice.len() {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) fn progress_and_get_chunk(&self, chunk_size: usize) -> Option<(usize, &'a [T])> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.slice.len())
                    .max(begin_idx);
                (begin_idx, &self.slice[begin_idx..end_idx])
            })
    }
}

impl<'a, T> ConcurrentIter for ConIterSliceRef<'a, T> {
    type Item = &'a T;

    type SeqIter = Skip<core::slice::Iter<'a, T>>;

    type ChunksIter<'i>
        = ChunksIterSliceRef<'i, 'a, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }

    fn next<N: Next<Self::Item>>(&self) -> Option<N> {
        self.progress_and_get_begin_idx(1)
            .map(|begin_idx| N::new(begin_idx, &self.slice[begin_idx]))
    }

    fn in_chunks(&self, chunk_size: usize) -> Self::ChunksIter<'_> {
        Self::ChunksIter::new(self, chunk_size)
    }
}
