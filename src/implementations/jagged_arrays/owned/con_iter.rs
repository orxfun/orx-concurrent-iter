use super::{
    chunk_puller::ChunkPullerJaggedOwned, into_iter::RawJaggedIterOwned, raw_jagged::RawJagged,
    slice_iter::RawJaggedSliceIterOwned,
};
use crate::{
    ConcurrentIter, ExactSizeConcurrentIter, implementations::jagged_arrays::indexer::JaggedIndexer,
};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Flattened concurrent iterator of a raw jagged array yielding owned elements.
///
/// Ensures that all elements are dropped regardless of whether they are iterated over or skipped.
/// Further, cleans up the allocations of the jagged array.
pub struct ConIterJaggedOwned<T, X>
where
    T: Send,
    X: JaggedIndexer,
{
    jagged: RawJagged<T, X>,
    counter: AtomicUsize,
}

unsafe impl<T: Send, X: JaggedIndexer> Sync for ConIterJaggedOwned<T, X> {}

impl<T, X> ConIterJaggedOwned<T, X>
where
    T: Send,
    X: JaggedIndexer,
{
    pub(crate) fn new(jagged: RawJagged<T, X>, begin: usize) -> Self {
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
    ) -> Option<(usize, RawJaggedSliceIterOwned<'_, T>)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.jagged.len())
                    .max(begin_idx);
                let slice = self.jagged.slice(begin_idx, end_idx);
                let iter = RawJaggedSliceIterOwned::new(slice);
                (begin_idx, iter)
            })
    }
}

impl<T, X> ConcurrentIter for ConIterJaggedOwned<T, X>
where
    T: Send,
    X: JaggedIndexer,
{
    type Item = T;

    type SequentialIter = RawJaggedIterOwned<T, X>;

    type ChunkPuller<'i>
        = ChunkPullerJaggedOwned<'i, T, X>
    where
        Self: 'i;

    fn into_seq_iter(mut self) -> Self::SequentialIter {
        let num_taken = self.counter.load(Ordering::Acquire).min(self.jagged.len());

        let jagged_to_drop = self.jagged.move_into_new(num_taken);

        RawJaggedIterOwned::new(jagged_to_drop)
    }

    fn skip_to_end(&self) {
        let current = self.counter.fetch_max(self.jagged.len(), Ordering::Acquire);
        let num_taken_before = current.min(self.jagged.len());
        let slice = self.jagged.slice_from(num_taken_before);
        let _iter = RawJaggedSliceIterOwned::new(slice);
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1).and_then(|idx| {
            // SAFETY: `counter` ensures that elements from each position is taken only once
            unsafe { self.jagged.take(idx) }
        })
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.progress_and_get_begin_idx(1).and_then(|idx| {
            // SAFETY: `counter` ensures that elements from each position is taken only once
            unsafe { self.jagged.take(idx).map(|value| (idx, value)) }
        })
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

impl<T, X> ExactSizeConcurrentIter for ConIterJaggedOwned<T, X>
where
    T: Send,
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.jagged.len().saturating_sub(num_taken)
    }
}

impl<T, X> Drop for ConIterJaggedOwned<T, X>
where
    T: Send,
    X: JaggedIndexer,
{
    fn drop(&mut self) {
        if self.jagged.num_taken().is_some() {
            let num_taken = self.counter.load(Ordering::Acquire);
            // SAFETY: `num_taken` elements are already taken out by the concurrent iterator.
            // Before dropping this iterator, this is set as the `num_taken` of the raw
            // jagged array which is responsible from dropping the elements and allocations.
            // This will ensure that these `num_taken` elements will not be attempted to be
            // dropped the second time.
            unsafe { self.jagged.set_num_taken(Some(num_taken)) };
        }
    }
}
