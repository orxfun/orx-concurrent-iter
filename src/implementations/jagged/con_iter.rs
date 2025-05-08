use super::raw_slices::RawSlices;
use core::sync::atomic::{AtomicUsize, Ordering};
use orx_iterable::Collection;

pub struct ConIterJagged<T>
where
    T: Send + Sync,
{
    slices: RawSlices<T>,
    begin: usize,
    len: usize,
    counter: AtomicUsize,
}

impl<T> ConIterJagged<T>
where
    T: Send + Sync,
{
    pub(crate) fn new(slices: RawSlices<T>, begin: usize, len: usize) -> Self {
        let total_len: usize = slices.iter().map(|x| x.len()).sum();
        let len = total_len - len;
        Self {
            slices,
            begin,
            len,
            counter: 0.into(),
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.len {
            true => Some(begin_idx),
            false => None,
        }
    }
}
