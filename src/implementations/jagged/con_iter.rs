use super::raw_jagged::RawJagged;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::fmt::Debug;

pub struct ConIterJagged<T, X>
where
    T: Send + Sync + Debug,
    X: Fn(usize) -> [usize; 2],
{
    slices: RawJagged<T, X>,
    begin: usize,
    counter: AtomicUsize,
}

impl<T, X> ConIterJagged<T, X>
where
    T: Send + Sync + Debug,
    X: Fn(usize) -> [usize; 2],
{
    pub(crate) fn new(slices: RawJagged<T, X>, begin: usize) -> Self {
        Self {
            slices,
            begin,
            counter: begin.into(),
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.slices.len() {
            true => Some(begin_idx),
            false => None,
        }
    }

    pub(super) fn progress_and_get_chunk_slices(&self, chunk_size: usize) -> Option<(usize, ())> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.slices.len())
                    .max(begin_idx);
                let range = begin_idx..end_idx;
                let iter = ();
                (begin_idx, iter)
            })
    }
}
