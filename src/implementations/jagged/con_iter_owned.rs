use super::{raw_jagged::RawJagged, raw_jagged_slice_iter_owned::RawJaggedSliceIterOwned};
use core::sync::atomic::{AtomicUsize, Ordering};
use std::fmt::Debug;

pub struct ConIterJaggedOwned<T, X>
where
    T: Send + Sync + Debug,
    X: Fn(usize) -> [usize; 2],
{
    jagged: RawJagged<T, X>,
    begin: usize,
    counter: AtomicUsize,
}

impl<T, X> ConIterJaggedOwned<T, X>
where
    T: Send + Sync + Debug,
    X: Fn(usize) -> [usize; 2],
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
