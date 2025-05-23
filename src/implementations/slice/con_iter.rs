use super::chunk_puller::ChunkPullerSlice;
use crate::{concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter};
use core::{
    iter::Skip,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Concurrent iterator of a slice.
///
/// It can be created by calling [`into_con_iter`] on a slice.
///
/// Alternatively, it can be created calling [`con_iter`] on the type
/// that owns the slice.
///
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
/// [`con_iter`]: crate::ConcurrentIterable::con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// // &[T]: IntoConcurrentIter
/// let vec = vec![0, 1, 2, 3];
/// let slice = &vec[1..3];
/// let con_iter = slice.into_con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
///
/// // Vec<T>: ConcurrentIterable
/// let vec = vec![1, 2];
/// let con_iter = vec.con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct ConIterSlice<'a, T>
where
    T: Send + Sync,
{
    slice: &'a [T],
    counter: AtomicUsize,
}

impl<T> Default for ConIterSlice<'_, T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self::new(&[])
    }
}

impl<'a, T> ConIterSlice<'a, T>
where
    T: Send + Sync,
{
    pub(crate) fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: 0.into(),
        }
    }

    pub(super) fn slice(&self) -> &'a [T] {
        self.slice
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.slice.len() {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) fn progress_and_get_slice(&self, chunk_size: usize) -> Option<(usize, &'a [T])> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.slice.len())
                    .max(begin_idx);
                (begin_idx, &self.slice[begin_idx..end_idx])
            })
    }
}

impl<'a, T> ConcurrentIter for ConIterSlice<'a, T>
where
    T: Send + Sync,
{
    type Item = &'a T;

    type SequentialIter = Skip<core::slice::Iter<'a, T>>;

    type ChunkPuller<'i>
        = ChunkPullerSlice<'i, 'a, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1)
            .map(|idx| &self.slice[idx])
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.progress_and_get_begin_idx(1)
            .map(|idx| (idx, &self.slice[idx]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_taken = self.counter.load(Ordering::Acquire);
        let remaining = self.slice.len().saturating_sub(num_taken);
        (remaining, Some(remaining))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterSlice<'_, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.slice.len().saturating_sub(num_taken)
    }
}
