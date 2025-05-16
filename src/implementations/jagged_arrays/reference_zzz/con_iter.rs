use super::{
    chunk_puller::ChunkPullerJaggedRef, raw_jagged_ref::RawJaggedRef,
    slice_iter::RawJaggedSliceIterRef,
};
use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    implementations::jagged_arrays::{JaggedIndexer, RawSlice},
};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;

/// Flattened concurrent iterator of a raw jagged array yielding references to elements.
pub struct ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
    jagged: RawJaggedRef<T, X>,
    counter: AtomicUsize,
    phantom: PhantomData<&'a ()>,
}

unsafe impl<'a, T, X> Sync for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
}

unsafe impl<'a, T, X> Send for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync + 'a,
{
}

impl<'a, T, X> ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
    pub(crate) fn new(jagged: RawJaggedRef<T, X>, begin: usize) -> Self {
        Self {
            jagged,
            counter: begin.into(),
            phantom: PhantomData,
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
    ) -> Option<(usize, RawJaggedSliceIterRef<'a, &'a Vec<RawSlice<T>>, T>)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.jagged.len())
                    .max(begin_idx);
                let slice = self.jagged.slice(begin_idx, end_idx);
                let iter = RawJaggedSliceIterRef::new(slice);
                // (begin_idx, Default::default())
                // (begin_idx, iter)
                todo!()
            })
    }
}

impl<'a, T, X> ConcurrentIter for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer + Send + Sync + 'a,
{
    type Item = &'a T;

    type SequentialIter = RawJaggedSliceIterRef<'a, Vec<RawSlice<T>>, T>;

    type ChunkPuller<'i>
        = ChunkPullerJaggedRef<'i, 'a, T, X>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let num_taken = self.counter.load(Ordering::Acquire).min(self.jagged.len());
        // let slice = self.jagged.into_slice_from(num_taken);
        // RawJaggedSliceIterRef::new(slice)
        todo!()
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

impl<'a, T, X> ExactSizeConcurrentIter for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer + Send + Sync + 'a,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.jagged.len().saturating_sub(num_taken)
    }
}
