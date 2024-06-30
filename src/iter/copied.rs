use super::{
    atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
    buffered::{
        buffered_chunk::BufferedChunk, buffered_iter::BufferedIter,
        copied_buffered_chunk::CopiedBufferedChunk,
    },
};
use crate::{ConcurrentIter, NextChunk};
use std::marker::PhantomData;

/// An concurrent iterator, backed with an atomic iterator, that copies the elements of an underlying iterator.
///
/// This `struct` is created by the `copied` method on the concurrent iterator.
pub struct Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T>,
{
    iter: A,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, A> Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T>,
{
    pub(crate) fn new(iter: A) -> Self {
        Self {
            iter,
            phantom: PhantomData,
        }
    }

    pub(crate) fn underlying_iter(&self) -> &A {
        &self.iter
    }
}

impl<'a, T, A> AtomicIter<T> for Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T>,
{
    #[inline(always)]
    fn counter(&self) -> &crate::AtomicCounter {
        self.iter.counter()
    }

    #[inline(always)]
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        self.iter.progress_and_get_begin_idx(number_to_fetch)
    }

    #[inline(always)]
    fn get(&self, item_idx: usize) -> Option<T> {
        self.iter.get(item_idx).copied()
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        self.iter.fetch_n(n).map(|x| NextChunk {
            begin_idx: x.begin_idx,
            values: x.values.copied(),
        })
    }

    fn early_exit(&self) {
        self.iter.early_exit()
    }
}

impl<'a, T, A> AtomicIterWithInitialLen<T> for Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T> + AtomicIterWithInitialLen<&'a T>,
{
    fn initial_len(&self) -> usize {
        self.iter.initial_len()
    }
}

unsafe impl<'a, T, A> Sync for Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T>,
{
}

unsafe impl<'a, T, A> Send for Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T>,
{
}

impl<'a, T, A> ConcurrentIter for Copied<'a, T, A>
where
    T: Send + Sync + Copy,
    A: AtomicIter<&'a T> + ConcurrentIter<Item = &'a T>,
{
    type Item = T;

    type BufferedIter = CopiedBufferedChunk<'a, T, A::BufferedIter>;

    type SeqIter = std::iter::Copied<A::SeqIter>;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_seq_iter().copied()
    }

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<crate::Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        self.fetch_n(chunk_size)
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    fn skip_to_end(&self) {
        self.early_exit()
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
    C: AtomicIter<&'a T>,
{
    /// Converts this concurrent iterator over references into another concurrent iterator yielding copies of the elements.
    fn copied(self) -> Copied<'a, T, C> {
        Copied::new(self.into())
    }
}

impl<'a, T, C> IntoCopied<'a, T, C> for C
where
    T: Send + Sync + Copy + 'a,
    C: AtomicIter<&'a T>,
{
}
