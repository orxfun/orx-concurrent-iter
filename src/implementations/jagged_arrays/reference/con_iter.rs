use super::{
    chunk_puller::ChunkPullerJaggedRef, raw_jagged_ref::RawJaggedRef,
    slice_iter::RawJaggedSliceIterRef,
};
use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    implementations::jagged_arrays::{JaggedIndexer, RawSlice, as_slice::AsSlice},
};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Flattened concurrent iterator of a raw jagged array yielding references to elements.
pub struct ConIterJaggedRef<'a, T, S, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer,
    S: AsSlice<T> + Send + Sync,
{
    jagged: RawJaggedRef<'a, T, S, X>,
    counter: AtomicUsize,
}

impl<'a, T, S, X> ConIterJaggedRef<'a, T, S, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer + Send + Sync,
    S: AsSlice<T> + Send + Sync,
{
    pub(crate) fn new(jagged: RawJaggedRef<'a, T, S, X>, begin: usize) -> Self {
        Self {
            jagged,
            counter: begin.into(),
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.jagged.len() {
            true => Some(begin_idx),
            false => None,
        }
    }

    pub(super) fn progress_and_get_iter(
        &self,
        chunk_size: usize,
    ) -> Option<(usize, RawJaggedSliceIterRef<'a, T, S, X>)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.jagged.len())
                    .max(begin_idx);
                let slice = self.jagged.jagged_slice(begin_idx, end_idx);
                let iter = RawJaggedSliceIterRef::new(slice);
                (begin_idx, iter)
            })
    }
}

impl<'a, T, S, X> ConcurrentIter for ConIterJaggedRef<'a, T, S, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer,
    S: AsSlice<T> + Send + Sync,
{
    type Item = &'a T;

    type SequentialIter = RawJaggedSliceIterRef<'a, T, S, X>;

    type ChunkPuller<'i>
        = ChunkPullerJaggedRef<'i, 'a, T, S, X>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let num_taken = self.counter.load(Ordering::Acquire).min(self.jagged.len());
        let flat_end = self.jagged.len();
        let slice = self.jagged.jagged_slice(num_taken, flat_end);
        RawJaggedSliceIterRef::new(slice)
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.jagged.len(), Ordering::Acquire);
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1)
            .and_then(|idx| self.jagged.get(idx))
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.progress_and_get_begin_idx(1)
            .and_then(|idx| self.jagged.get(idx).map(|value| (idx, value)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_taken = self.counter.load(Ordering::Acquire);
        let remaining = self.jagged.len().saturating_sub(num_taken);
        (remaining, Some(remaining))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<'a, T, S, X> ExactSizeConcurrentIter for ConIterJaggedRef<'a, T, S, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer,
    S: AsSlice<T> + Send + Sync,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.jagged.len().saturating_sub(num_taken)
    }
}
