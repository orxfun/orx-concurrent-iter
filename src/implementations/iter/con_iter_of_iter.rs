use super::{
    super::mut_handle::{AtomicState, MutHandle, COMPLETED},
    iter_cell::IterCell,
};
use crate::{
    chunk_puller::DoNothingChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumeration, Regular},
};
use core::marker::PhantomData;

pub struct ConIterOfIter<I, T, E = Regular>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    iter: IterCell<T, I>,
    initial_len: Option<usize>,
    state: AtomicState,
    phantom: PhantomData<E>,
}

unsafe impl<I: Iterator<Item = T>, T: Send + Sync, E: Enumeration> Sync for ConIterOfIter<I, T, E> {}

unsafe impl<I: Iterator<Item = T>, T: Send + Sync, E: Enumeration> Send for ConIterOfIter<I, T, E> {}

impl<I, T, E> Default for ConIterOfIter<I, T, E>
where
    T: Send + Sync,
    I: Iterator<Item = T> + Default,
    E: Enumeration,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I, T, E> ConIterOfIter<I, T, E>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    pub(super) fn new(iter: I) -> Self {
        let initial_len = match iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(lower),
            _ => None,
        };

        Self {
            iter: iter.into(),
            initial_len,
            state: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<I, T, E> ConcurrentIter<E> for ConIterOfIter<I, T, E>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    type Item = T;

    type SeqIter = I;

    type ChunkPuller<'i>
        = DoNothingChunkPuller<E, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_inner()
    }

    fn skip_to_end(&self) {
        todo!()
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        todo!()
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}
