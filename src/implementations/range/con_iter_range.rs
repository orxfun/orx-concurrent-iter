use super::chunk_puller_range::ChunkPullerRange;
use crate::concurrent_iterator::ConcurrentIterator;
use core::{
    marker::PhantomData,
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterRange<T> {
    begin: usize,
    len: usize,
    counter: AtomicUsize,
    phantom: PhantomData<T>,
}

impl<T> Default for ConIterRange<T> {
    fn default() -> Self {
        Self {
            begin: Default::default(),
            len: Default::default(),
            counter: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<T> ConIterRange<T>
where
    T: Send + Sync + From<usize>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    pub(super) fn new(range: Range<usize>) -> Self {
        Self {
            begin: range.start,
            len: range.end - range.start,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.len {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) fn progress_and_get_range(&self, chunk_size: usize) -> Option<(usize, T, T)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size).min(self.len).max(begin_idx);
                let begin = self.begin + begin_idx;
                let end = self.begin + end_idx;
                (begin_idx, begin.into(), end.into())
            })
    }
}

impl<T> ConcurrentIterator for ConIterRange<T>
where
    T: Send + Sync + From<usize>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type SequentialIter = Range<T>;

    type ChunkPuller<'i>
        = ChunkPullerRange<'i, Self::Item>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let current = self.counter.load(Ordering::Acquire);
        let begin = T::from(self.begin + current);
        let end = T::from(self.begin + self.len);
        begin..end
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.len, Ordering::Acquire);
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1)
            .map(|idx| T::from(self.begin + idx))
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.progress_and_get_begin_idx(1)
            .map(|idx| (idx, T::from(self.begin + idx)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_taken = self.counter.load(Ordering::Acquire);
        let remaining = self.len.saturating_sub(num_taken);
        (remaining, Some(remaining))
    }
}
