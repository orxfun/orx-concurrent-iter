use super::{
    chunk_puller_owned::ChunkPullerJaggedOwned, raw_jagged::RawJagged,
    raw_jagged_iter_owned::RawJaggedIterOwned,
    raw_jagged_slice_iter_owned::RawJaggedSliceIterOwned,
    raw_jagged_slice_iter_ref::RawJaggedSliceIterRef,
};
use crate::{ConcurrentIter, ExactSizeConcurrentIter};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
    jagged: &'a RawJagged<T, X>,
    counter: AtomicUsize,
}

impl<'a, T, X> ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
    pub(crate) fn new(jagged: &'a RawJagged<T, X>, begin: usize) -> Self {
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
    ) -> Option<(usize, RawJaggedSliceIterRef<'a, T>)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.jagged.len())
                    .max(begin_idx);
                let slice = self.jagged.slice(begin_idx, end_idx);
                let iter = slice.into_iter_ref();
                (begin_idx, iter)
            })
    }
}
