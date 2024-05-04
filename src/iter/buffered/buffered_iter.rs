use super::buffered_chunk::BufferedChunk;
use crate::{iter::atomic_iter::AtomicIter, NextChunk};
use std::marker::PhantomData;

pub struct BufferedIter<'a, T, B>
where
    T: Send + Sync,
    B: BufferedChunk<T>,
{
    buffered_iter: B,
    atomic_iter: &'a B::ConIter,
    phantom: PhantomData<T>,
}

impl<'a, T, B> BufferedIter<'a, T, B>
where
    T: Send + Sync,
    B: BufferedChunk<T>,
{
    pub(crate) fn new(buffered_iter: B, atomic_iter: &'a B::ConIter) -> Self {
        Self {
            buffered_iter,
            atomic_iter,
            phantom: Default::default(),
        }
    }

    #[allow(clippy::unwrap_used, clippy::unwrap_in_result, clippy::question_mark)]
    pub fn next(&mut self) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T> + '_>> {
        self.atomic_iter
            .progress_and_get_begin_idx(self.buffered_iter.chunk_size())
            .and_then(|begin_idx| {
                self.buffered_iter
                    .pull(self.atomic_iter, begin_idx)
                    .map(|values| NextChunk { begin_idx, values })
            })
    }
}
