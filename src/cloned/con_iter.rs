use super::chunk_puller::ClonedChunkPuller;
use crate::concurrent_iter::ConcurrentIter;
use core::{iter::Cloned, marker::PhantomData};

/// A concurrent iterator which clones all elements.
///
/// This is useful when you have an iterator over `&T`, but you need an iterator over `T`.
///
/// Cloned concurrent iterator can be created by calling [`cloned`] on a concurrent iterator.
///
/// [`cloned`]: crate::ConcurrentIter::cloned
pub struct ConIterCloned<'a, I, T>
where
    T: Send + Sync + Clone,
    I: ConcurrentIter<Item = &'a T>,
{
    con_iter: I,
    phantom: PhantomData<&'a T>,
}

impl<'a, I, T> ConIterCloned<'a, I, T>
where
    T: Send + Sync + Clone,
    I: ConcurrentIter<Item = &'a T>,
{
    pub(crate) fn new(con_iter: I) -> Self {
        Self {
            con_iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T> ConcurrentIter for ConIterCloned<'a, I, T>
where
    T: Send + Sync + Clone,
    I: ConcurrentIter<Item = &'a T>,
{
    type Item = T;

    type SequentialIter = Cloned<I::SequentialIter>;

    type ChunkPuller<'i>
        = ClonedChunkPuller<'a, T, I::ChunkPuller<'i>>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        self.con_iter.into_seq_iter().cloned()
    }

    fn skip_to_end(&self) {
        self.con_iter.skip_to_end()
    }

    fn next(&self) -> Option<Self::Item> {
        self.con_iter.next().cloned()
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.con_iter.next_with_idx().map(|(i, x)| (i, x.clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.con_iter.size_hint()
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        self.con_iter.chunk_puller(chunk_size).into()
    }
}
