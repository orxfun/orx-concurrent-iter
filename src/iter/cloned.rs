use super::buffered::{
    buffered_chunk::BufferedChunk, buffered_iter::BufferedIter,
    cloned_buffered_chunk::ClonedBufferedChunk,
};
use crate::{ConcurrentIter, NextChunk};
use std::marker::PhantomData;

/// An concurrent iterator, backed with an atomic iterator, that clones the elements of an underlying iterator.
///
/// This `struct` is created by the `cloned` method on the concurrent iterator.
pub struct Cloned<'a, T, C>
where
    T: Send + Sync + Clone,
    C: ConcurrentIter<Item = &'a T>,
{
    iter: C,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, C> Cloned<'a, T, C>
where
    T: Send + Sync + Clone,
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

unsafe impl<'a, T, A> Sync for Cloned<'a, T, A>
where
    T: Send + Sync + Clone,
    A: ConcurrentIter<Item = &'a T>,
{
}

unsafe impl<'a, T, C> Send for Cloned<'a, T, C>
where
    T: Send + Sync + Clone,
    C: ConcurrentIter<Item = &'a T>,
{
}

impl<'a, T, C> ConcurrentIter for Cloned<'a, T, C>
where
    T: Send + Sync + Clone,
    C: ConcurrentIter<Item = &'a T>,
{
    type Item = T;

    type BufferedIter = ClonedBufferedChunk<'a, T, C::BufferedIter>;

    type SeqIter = std::iter::Cloned<C::SeqIter>;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_seq_iter().cloned()
    }

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<crate::Next<Self::Item>> {
        self.iter.next_id_and_value().map(|x| x.cloned())
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        self.iter.next_chunk(chunk_size).map(|x| x.cloned())
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

/// Trait converting a concurrent iterator yielding &T to one yielding T by cloning elements.
pub trait IntoCloned<'a, T, C>
where
    Self: Into<C>,
    T: Send + Sync + Clone + 'a,
    C: ConcurrentIter<Item = &'a T>,
{
    /// Converts this concurrent iterator over references into another concurrent iterator yielding clones of the elements.
    fn cloned(self) -> Cloned<'a, T, C> {
        Cloned::new(self.into())
    }
}

impl<'a, T, C> IntoCloned<'a, T, C> for C
where
    T: Send + Sync + Clone + 'a,
    C: ConcurrentIter<Item = &'a T>,
{
}
