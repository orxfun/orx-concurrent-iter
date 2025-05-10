use super::{
    chunk_puller_owned::ChunkPullerJaggedOwned, raw_jagged::RawJagged,
    raw_jagged_iter_owned::RawJaggedIterOwned,
    raw_jagged_slice_iter_owned::RawJaggedSliceIterOwned,
};
use crate::ConcurrentIter;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ConIterJaggedOwned<T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
    jagged: RawJagged<T, X>,
    begin: usize,
    counter: AtomicUsize,
}

unsafe impl<T, X> Sync for ConIterJaggedOwned<T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
}

unsafe impl<T, X> Send for ConIterJaggedOwned<T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
}

impl<T, X> ConIterJaggedOwned<T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
    pub(crate) fn new(jagged: RawJagged<T, X>, begin: usize) -> Self {
        Self {
            jagged,
            begin,
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
    ) -> Option<(usize, RawJaggedSliceIterOwned<T>)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.jagged.len())
                    .max(begin_idx);
                let slice = self.jagged.slice(begin_idx, end_idx);
                let iter = slice.into_iter_owned();
                (begin_idx, iter)
            })
    }
}

impl<T, X> ConcurrentIter for ConIterJaggedOwned<T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
    type Item = T;

    type SequentialIter = RawJaggedIterOwned<T, X>;

    type ChunkPuller<'i>
        = ChunkPullerJaggedOwned<'i, T, X>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let num_taken = self.counter.load(Ordering::Acquire).min(self.jagged.len());
        RawJaggedIterOwned::new(self.jagged, num_taken)
    }

    fn skip_to_end(&self) {
        let current = self.counter.fetch_max(self.jagged.len(), Ordering::Acquire);
        let num_taken_before = current.min(self.jagged.len());
        let slice = self.jagged.slice_from(num_taken_before);
        let _iter = slice.into_iter_owned();
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1)
            .and_then(|idx| self.jagged.take(idx))
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
