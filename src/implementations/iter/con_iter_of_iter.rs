use super::{
    chunk_puller_iter::ChunkPullerOfIter,
    iter_cell::IterCell,
    mut_handle::{AtomicState, MutHandle, COMPLETED},
};
use crate::{
    concurrent_iter::{ConcurrentIter, ConcurrentIterEnum},
    enumeration::{Element, Enumeration, Regular},
};
use core::{marker::PhantomData, sync::atomic::Ordering};

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

    fn get_handle(&self) -> Option<MutHandle<'_>> {
        MutHandle::get_handle(&self.state)
    }

    /// Pulls and writes chunk-size (`buffer.len()`) elements from the iterator into the given `buffer` starting from position 0.
    ///
    /// Returns the pair of (begin_idx, num_taken):
    ///
    /// * begin_idx: index of the first taken item.
    /// * num_taken: number of items pulled from the iterator; the method tries to pull `buffer.len()` items, however, might stop
    ///   early if the iterator is completely consumed.
    pub(super) fn next_chunk_to_buffer(&self, buffer: &mut [Option<T>]) -> (usize, usize) {
        self.get_handle()
            .map(|handle| self.iter.next_chunk_to_buffer(handle, buffer))
            .unwrap_or((0, 0))
    }
}

impl<I, T, E> ConcurrentIterEnum<E, T> for ConIterOfIter<I, T, E>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    type EnumerationOf<E2>
        = ConIterOfIter<I, T, E2>
    where
        E2: Enumeration;

    fn into_enumeration_of<E2: Enumeration>(self) -> Self::EnumerationOf<E2> {
        ConIterOfIter {
            iter: self.iter,
            initial_len: self.initial_len,
            state: self.state,
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
        = ChunkPullerOfIter<'i, I, T, E>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_inner()
    }

    fn skip_to_end(&self) {
        self.state.store(COMPLETED, Ordering::SeqCst);
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        self.get_handle()
            .and_then(|handle| self.iter.next::<E>(handle))
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}
