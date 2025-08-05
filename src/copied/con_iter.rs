use super::chunk_puller::CopiedChunkPuller;
use crate::{ExactSizeConcurrentIter, concurrent_iter::ConcurrentIter};
use core::{iter::Copied, marker::PhantomData};

/// A concurrent iterator which copies all elements.
///
/// This is useful when you have an iterator over `&T`, but you need an iterator over `T`.
///
/// Copied concurrent iterator can be created by calling [`copied`] on a concurrent iterator.
///
/// [`copied`]: crate::ConcurrentIter::copied
pub struct ConIterCopied<'a, I, T>
where
    T: Copy,
    I: ConcurrentIter<Item = &'a T>,
{
    con_iter: I,
    phantom: PhantomData<&'a T>,
}

unsafe impl<'a, I, T> Sync for ConIterCopied<'a, I, T>
where
    T: Copy + Send,
    I: ConcurrentIter<Item = &'a T>,
{
}

impl<'a, I, T> ConIterCopied<'a, I, T>
where
    T: Copy,
    I: ConcurrentIter<Item = &'a T>,
{
    pub(crate) fn new(con_iter: I) -> Self {
        Self {
            con_iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T> ConcurrentIter for ConIterCopied<'a, I, T>
where
    T: Copy + Send,
    I: ConcurrentIter<Item = &'a T>,
{
    type Item = T;

    type SequentialIter = Copied<I::SequentialIter>;

    type ChunkPuller<'i>
        = CopiedChunkPuller<'a, T, I::ChunkPuller<'i>>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        self.con_iter.into_seq_iter().copied()
    }

    fn skip_to_end(&self) {
        self.con_iter.skip_to_end()
    }

    fn next(&self) -> Option<Self::Item> {
        self.con_iter.next().copied()
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.con_iter.next_with_idx().map(|(i, x)| (i, *x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.con_iter.size_hint()
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        self.con_iter.chunk_puller(chunk_size).into()
    }
}

impl<'a, I, T> ExactSizeConcurrentIter for ConIterCopied<'a, I, T>
where
    T: Copy + Send,
    I: ExactSizeConcurrentIter<Item = &'a T>,
{
    fn len(&self) -> usize {
        self.con_iter.len()
    }
}
