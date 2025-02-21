use super::{
    super::mut_handle::{AtomicState, MutHandle, COMPLETED},
    mut_iter::MutIter,
    num_taken::NumTaken,
};
use crate::{
    chunk_puller::{ChunkPuller, DoNothingChunkPuller},
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumeration, Regular},
};
use core::{cell::UnsafeCell, marker::PhantomData, sync::atomic::Ordering};

pub struct ConIterOfIter<I, T, E = Regular>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    iter: MutIter<T, I>,
    num_taken: NumTaken,
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
            num_taken: 0.into(),
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

    type SeqIter = core::iter::Empty<T>;

    type ChunkPuller<'i>
        = DoNothingChunkPuller<E, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter {
        todo!()
    }

    fn skip_to_end(&self) {
        todo!()
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        todo!()
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
