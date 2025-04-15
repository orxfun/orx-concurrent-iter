use super::chunk_puller::EnumeratedChunkPuller;
use crate::ConcurrentIter;

/// An enumerated version of a concurrent iterator which additionally yields
/// the index of elements in the source collection.
///
/// It can be created by calling [`enumerate`] on a concurrent iterator.
///
/// [`enumerate`]: crate::ConcurrentIter::enumerate
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec!['x', 'y'];
///
/// let con_iter = vec.con_iter().enumerate();
/// assert_eq!(con_iter.next(), Some((0, &'x')));
/// assert_eq!(con_iter.next(), Some((1, &'y')));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct Enumerate<I>
where
    I: ConcurrentIter,
{
    iter: I,
}

impl<I> Enumerate<I>
where
    I: ConcurrentIter,
{
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> ConcurrentIter for Enumerate<I>
where
    I: ConcurrentIter,
{
    type Item = (usize, I::Item);

    type SequentialIter = core::iter::Enumerate<I::SequentialIter>;

    type ChunkPuller<'i>
        = EnumeratedChunkPuller<I::ChunkPuller<'i>>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        self.iter.into_seq_iter().enumerate()
    }

    fn skip_to_end(&self) {
        self.iter.skip_to_end();
    }

    fn next(&self) -> Option<Self::Item> {
        self.iter.next_with_idx()
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.iter.next_with_idx().map(|(i, x)| (i, (i, x)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self.iter.chunk_puller(chunk_size))
    }
}
