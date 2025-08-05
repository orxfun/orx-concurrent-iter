use super::{
    chunk_puller::ChunkPullerOfIter,
    iter_cell::IterCell,
    mut_handle::{AtomicState, COMPLETED, MutHandle},
};
use crate::{concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter};
use core::sync::atomic::Ordering;

/// Concurrent iterator of a any generic type implementing a
/// regular [`Iterator`].
///
/// It can be created by calling [`iter_into_con_iter`] on any iterator.
///
/// This iterator has a fundamental difference from all other concurrent iterators in the following:
///
/// * Concurrent iterators in general allow for concurrent access to different elements of the
///   source code without blocking each other;
/// * however, concurrent iterator of a generic iterator requires to serialize generation of elements
///   which might lead pulling threads to wait each other.
///
/// This has the following implications:
///
/// * Whenever possible, it is better to create the concurrent iterator on the concrete type rather
///   than the generic iterator.
/// * Still, the transformed concurrent iterator allows for a very convenient way to safely share the
///   iterator among multiple threads, simply by a shared reference.
/// * Furthermore, for programs where the task performed on each element of the iterator is
///   large enough, the overhead might be considered tolerable.
///
/// [`iter_into_con_iter`]: crate::IterIntoConcurrentIter::iter_into_con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let num_threads = 4;
///
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
///
/// // an arbitrary iterator
/// let iter = data
///     .into_iter()
///     .filter(|x| !x.starts_with('3'))
///     .map(|x| format!("{x}!"));
///
/// // converted into a concurrent iterator and shared with multiple threads
/// let con_iter = iter.iter_into_con_iter();
///
/// let process = |_x: String| { /* assume actual work */ };
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             while let Some(value) = con_iter.next() {
///                 assert!(!value.starts_with('3') && value.ends_with('!'));
///                 process(value);
///             }
///         });
///     }
/// });
/// ```
pub struct ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send,
{
    iter: IterCell<I::Item, I>,
    state: AtomicState,
}

unsafe impl<I: Iterator> Sync for ConIterOfIter<I> where I::Item: Send {}

unsafe impl<I: Iterator> Send for ConIterOfIter<I> where I::Item: Send {}

impl<I> Default for ConIterOfIter<I>
where
    I: Iterator + Default,
    I::Item: Send,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I> ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send,
{
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter: iter.into(),
            state: 0.into(),
        }
    }

    fn get_handle(&self) -> Option<MutHandle<'_>> {
        MutHandle::get_handle(&self.state)
    }

    /// Pulls and writes chunk-size (`buffer.len()`) elements from the iterator into the given `buffer` starting from position 0.
    ///
    /// Returns the pair of (begin_idx, num_taken):
    ///
    /// * begin_idx: index of the first taken item.
    /// * num_taken: number of items pulled from the iterator; the method tries to pull `buffer.len()` items, however, might stop
    ///   early if the iterator is completely consumed.
    pub(super) fn next_chunk_to_buffer(&self, buffer: &mut [Option<I::Item>]) -> (usize, usize) {
        self.get_handle()
            .map(|handle| self.iter.next_chunk_to_buffer(handle, buffer))
            .unwrap_or((0, 0))
    }
}

impl<I> ConcurrentIter for ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send,
{
    type Item = I::Item;

    type SequentialIter = I;

    type ChunkPuller<'i>
        = ChunkPullerOfIter<'i, I>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        self.iter.into_inner()
    }

    fn skip_to_end(&self) {
        self.state.store(COMPLETED, Ordering::SeqCst);
    }

    fn next(&self) -> Option<Self::Item> {
        self.get_handle().and_then(|h| self.iter.next(h))
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.get_handle().and_then(|h| self.iter.next_with_idx(h))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.get_handle() {
            Some(h) => self.iter.size_hint(h),
            None => (0, Some(0)),
        }
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<I> ExactSizeConcurrentIter for ConIterOfIter<I>
where
    I: ExactSizeIterator,
    I::Item: Send,
{
    fn len(&self) -> usize {
        match self.get_handle() {
            Some(h) => self.iter.len(h),
            None => 0,
        }
    }
}
