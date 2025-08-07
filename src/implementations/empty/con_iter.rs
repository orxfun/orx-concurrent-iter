use super::chunk_puller::ChunkPullerEmpty;
use crate::{ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter};
use core::marker::PhantomData;

/// An empty concurrent iterator which does not yield any elements.
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let con_iter = iter::empty::<String>();
/// assert_eq!(con_iter.next(), None);
///
/// // or
///
/// let con_iter = implementations::ConIterEmpty::<String>::new();
/// assert_eq!(con_iter.next(), None);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ConIterEmpty<T> {
    phantom: PhantomData<T>,
}

unsafe impl<T> Sync for ConIterEmpty<T> {}

impl<T> Default for ConIterEmpty<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConIterEmpty<T> {
    /// Creates a new empty concurrent iterator with no elements.
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> ConcurrentIter for ConIterEmpty<T>
where
    T: Send,
{
    type Item = T;

    type SequentialIter = core::iter::Empty<T>;

    type ChunkPuller<'i>
        = ChunkPullerEmpty<'i, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        core::iter::empty()
    }

    fn skip_to_end(&self) {}

    fn next(&self) -> Option<Self::Item> {
        None
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterEmpty<T>
where
    T: Send,
{
    fn len(&self) -> usize {
        0
    }
}
