use super::{
    super::mut_handle::{AtomicState, MutHandle, COMPLETED},
    mut_iter::MutIter,
    num_taken::NumTaken,
};
use crate::{
    chunk_puller::DoNothingChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumeration, Regular},
};
use core::{cell::UnsafeCell, sync::atomic::Ordering};

pub struct ConIterOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: MutIter<T, I>,
    num_taken: NumTaken,
    initial_len: Option<usize>,
    state: AtomicState,
}

unsafe impl<I: Iterator<Item = T>, T: Send + Sync> Sync for ConIterOfIter<I, T> {}

unsafe impl<I: Iterator<Item = T>, T: Send + Sync> Send for ConIterOfIter<I, T> {}

impl<I, T> Default for ConIterOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T> + Default,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I, T> ConIterOfIter<I, T>
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
            num_taken: 0.into(),
            state: 0.into(),
        }
    }
}
