use super::chunk_puller::ChunkPullerRange;
use crate::{concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter};
use core::{
    marker::PhantomData,
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Concurrent iterator of a [`Range`].
///
/// It can be created by calling [`into_con_iter`] on a range.
///
/// [`Range`]: core::ops::Range
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let range = 1..3;
/// let con_iter = range.into_con_iter();
/// assert_eq!(con_iter.next(), Some(1));
/// assert_eq!(con_iter.next(), Some(2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct ConIterRange<T> {
    begin: usize,
    len: usize,
    counter: AtomicUsize,
    phantom: PhantomData<T>,
}

unsafe impl<T> Sync for ConIterRange<T>
where
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
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
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    pub(super) fn new(range: Range<T>) -> Self {
        let begin: usize = range.start.into();
        let end: usize = range.end.into();
        let len = end.saturating_sub(begin);
        Self {
            begin,
            len,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    pub(super) fn begin(&self) -> usize {
        self.begin
    }

    pub(super) fn initial_len(&self) -> usize {
        self.len
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

impl<T> ConcurrentIter for ConIterRange<T>
where
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
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

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        (self, chunk_size).into()
    }
}

impl<T> ExactSizeConcurrentIter for ConIterRange<T>
where
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.len.saturating_sub(num_taken)
    }
}
