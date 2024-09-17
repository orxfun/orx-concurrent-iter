use super::buffered::copied_buffered_chunk::CopiedBufferedChunk;
use crate::{ConcurrentIter, ConcurrentIterX, NextChunk};
use core::marker::PhantomData;

/// An concurrent iterator, backed with an atomic iterator, that copies the elements of an underlying iterator.
///
/// This `struct` is created by the `copied` method on the concurrent iterator.
pub struct Copied<'a, T, C>
where
    T: Send + Sync + Copy,
{
    iter: C,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, C> Copied<'a, T, C>
where
    T: Send + Sync + Copy,
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

unsafe impl<'a, T, C> Sync for Copied<'a, T, C> where T: Send + Sync + Copy {}

unsafe impl<'a, T, C> Send for Copied<'a, T, C> where T: Send + Sync + Copy {}

impl<'a, T, C> ConcurrentIterX for Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIterX<Item = &'a T>,
{
    type Item = T;

    type SeqIter = core::iter::Copied<C::SeqIter>;

    type BufferedIterX = CopiedBufferedChunk<'a, T, C::BufferedIterX>;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_seq_iter().copied()
    }

    fn next_chunk_x(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = Self::Item>> {
        self.iter.next_chunk_x(chunk_size).map(|x| x.copied())
    }

    fn next(&self) -> Option<Self::Item> {
        self.iter.next().copied()
    }

    fn skip_to_end(&self) {
        self.iter.skip_to_end()
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    #[inline(always)]
    fn try_get_initial_len(&self) -> Option<usize> {
        self.iter.try_get_initial_len()
    }
}

impl<'a, T, C> ConcurrentIter for Copied<'a, T, C>
where
    T: Send + Sync + Copy,
    C: ConcurrentIter<Item = &'a T>,
{
    type BufferedIter = CopiedBufferedChunk<'a, T, C::BufferedIter>;

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
