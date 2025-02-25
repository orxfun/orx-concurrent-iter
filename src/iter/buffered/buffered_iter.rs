use super::buffered_chunk::{BufferedChunk, BufferedChunkX};
use crate::NextChunk;
use core::marker::PhantomData;

pub struct BufferedIter<'a, T, B>
where
    T: Send + Sync,
    B: BufferedChunkX<T>,
{
    buffered_iter: B,
    iter: &'a B::ConIter,
    phantom: PhantomData<T>,
}

impl<'a, T, B> BufferedIter<'a, T, B>
where
    T: Send + Sync,
    B: BufferedChunkX<T>,
{
    pub(crate) fn new(buffered_iter: B, atomic_iter: &'a B::ConIter) -> Self {
        assert!(
            buffered_iter.chunk_size() > 0,
            "Chunk size must be positive."
        );

        Self {
            buffered_iter,
            iter: atomic_iter,
            phantom: PhantomData,
        }
    }

    #[allow(clippy::unwrap_used, clippy::unwrap_in_result, clippy::question_mark)]
    pub fn next_x(&mut self) -> Option<impl ExactSizeIterator<Item = T> + '_> {
        self.buffered_iter.pull_x(self.iter)
    }
}

impl<T, B> BufferedIter<'_, T, B>
where
    T: Send + Sync,
    B: BufferedChunk<T>,
{
    #[allow(clippy::unwrap_used, clippy::unwrap_in_result, clippy::question_mark)]
    pub fn next(&mut self) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T> + '_>> {
        self.buffered_iter.pull(self.iter)
    }
}
