use super::mut_handle::{AtomicState, MutHandle, AVAILABLE, COMPLETED};
use crate::{
    chunk_puller::DoNothingChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumeration, Regular},
};
use core::{cell::UnsafeCell, sync::atomic::Ordering};

pub struct ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: UnsafeCell<I>,
    initial_len: Option<usize>,
    state: AtomicState,
}

// TODO: drop when Vec.into_iter() for instance

impl<I, T> Default for ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T> + Default,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I, T> ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    pub(super) fn new(iter: I) -> Self {
        let initial_len = match iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(lower),
            _ => None,
        };

        Self {
            iter: iter.into(),
            initial_len,
            state: AVAILABLE.into(),
        }
    }
}

impl<I, T> ConcurrentIter<Regular> for ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    type Item = T;

    type SeqIter = I;

    type ChunkPuller<'i>
        = DoNothingChunkPuller<Regular, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.iter.into_inner()
    }

    fn skip_to_end(&self) {
        self.state.store(COMPLETED, Ordering::SeqCst);
    }

    fn next(&self) -> Option<<<Regular as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        MutHandle::get_handle(&self.state).and_then(|mut handle| {
            // SAFETY: no other thread has the handle
            let next = unsafe { &mut *self.iter.get() }.next();
            if next.is_none() {
                handle.set_target_to_completed();
            }
            next
        })
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
