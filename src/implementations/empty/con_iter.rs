use super::chunk_puller::ChunkPullerEmpty;
use crate::{exact_size_concurrent_iter::ExactSizeConcurrentIter, ConcurrentIter};
use core::marker::PhantomData;

pub struct ConIterEmpty<T: Send + Sync> {
    phantom: PhantomData<T>,
}

unsafe impl<T: Send + Sync> Sync for ConIterEmpty<T> {}

unsafe impl<T: Send + Sync> Send for ConIterEmpty<T> {}

impl<T> Default for ConIterEmpty<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConIterEmpty<T>
where
    T: Send + Sync,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> ConcurrentIter for ConIterEmpty<T>
where
    T: Send + Sync,
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
    T: Send + Sync,
{
    fn len(&self) -> usize {
        0
    }
}
