use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

use crate::{
    chunk_puller::DoNothingChunkPuller, concurrent_iter::ConcurrentIter, enumeration::Regular,
};

type State = u8;
const AVAILABLE: State = 0;
const IS_MUTATING: State = 1;
const COMPLETED: State = 2;

pub struct ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: UnsafeCell<I>,
    initial_len: Option<usize>,
    counter: AtomicUsize,
    is_mutating: AtomicU8,
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
            counter: 0.into(),
            is_mutating: AVAILABLE.into(),
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        match number_to_fetch {
            0 => None,
            _ => {
                let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
                loop {
                    match self.try_get_handle() {
                        Ok(()) => return Some(begin_idx),
                        Err(COMPLETED) => return None,
                        _ => {}
                    }
                }
            }
        }
    }

    fn try_get_handle(&self) -> Result<(), State> {
        self.is_mutating
            .compare_exchange(AVAILABLE, IS_MUTATING, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| ())
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
        todo!()
    }

    fn next(&self) -> Option<<<Regular as crate::enumeration::Enumeration>::Element as crate::enumeration::Element>::ElemOf<Self::Item>>{
        todo!()
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
