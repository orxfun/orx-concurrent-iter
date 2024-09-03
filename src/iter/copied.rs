use super::buffered::{
    buffered_chunk::BufferedChunk, buffered_iter::BufferedIter,
    copied_buffered_chunk::CopiedBufferedChunk,
};
use crate::{ConcurrentIter, NextChunk};
use std::marker::PhantomData;

/// An concurrent iterator, backed with an atomic iterator, that copies the elements of an underlying iterator.
///
/// This `struct` is created by the `copied` method on the concurrent iterator.
pub struct Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIter<Item = &'a T>,
{
    iter: C,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, C> Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIter<Item = &'a T>,
{
    pub(crate) fn new(iter: C) -> Self {
        Self {
            iter,
            phantom: PhantomData,
        }
    }

    pub(crate) fn underlying_iter(&self) -> &C {
        &self.iter
    }
}

unsafe impl<'a, T, C> Sync for Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIter<Item = &'a T>,
{
}

unsafe impl<'a, T, C> Send for Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIter<Item = &'a T>,
{
}

impl<'a, T, C> ConcurrentIter for Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIter<Item = &'a T>,
{
    type Item = T;

    type BufferedIter = CopiedBufferedChunk<'a, T, C::BufferedIter>;

    type SeqIter = std::iter::Copied<C::SeqIter>;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_seq_iter().copied()
    }

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<crate::Next<Self::Item>> {
        self.iter.next_id_and_value().map(|x| x.copied())
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        self.iter.next_chunk(chunk_size).map(|x| x.copied())
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    fn skip_to_end(&self) {
        self.iter.skip_to_end()
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }
}

/// Trait converting a concurrent iterator yielding &T to one yielding T by copying elements.
pub trait IntoCopied<'a, T, C>
where
    Self: Into<C>,
    T: Send + Sync + Copy + 'a,
    C: ConcurrentIter<Item = &'a T>,
{
    /// Converts this concurrent iterator over references into another concurrent iterator yielding copies of the elements.
    fn copied(self) -> Copied<'a, T, C> {
        Copied::new(self.into())
    }
}

impl<'a, T, C> IntoCopied<'a, T, C> for C
where
    T: Send + Sync + Copy + 'a,
    C: ConcurrentIter<Item = &'a T>,
{
}
