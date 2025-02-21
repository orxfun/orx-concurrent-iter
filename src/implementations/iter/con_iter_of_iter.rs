use super::mut_handle::{AtomicState, MutHandle, COMPLETED};
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
    pub(super) iter: UnsafeCell<I>,
    initial_len: Option<usize>,
    state: AtomicState,
}

unsafe impl<I: Iterator<Item = T>, T: Send + Sync> Sync for ConIterXOfIter<I, T> {}

unsafe impl<I: Iterator<Item = T>, T: Send + Sync> Send for ConIterXOfIter<I, T> {}

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
            state: 0.into(),
        }
    }

    pub(super) fn get_handle(&self, num_to_pull: usize) -> Option<MutHandle<'_>> {
        MutHandle::get_handle(&self.state, num_to_pull)
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
        None
        // self.get_handle().and_then(|mut handle| {
        //     // SAFETY: no other thread has the handle
        //     let next = unsafe { &mut *self.iter.get() }.next();
        //     if next.is_none() {
        //         handle.set_target_to_completed();
        //     }
        //     next
        // })
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}
