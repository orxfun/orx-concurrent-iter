use super::buffered_chunk::{BufferedChunk, BufferedChunkX};
use crate::{iter::copied::Copied, NextChunk};
use core::marker::PhantomData;

pub struct CopiedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Copy,
    C: BufferedChunkX<&'a T>,
{
    chunk: C,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, C> BufferedChunkX<T> for CopiedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Copy,
    C: BufferedChunkX<&'a T>,
{
    type ConIter = Copied<'a, T, C::ConIter>;

    fn new(chunk_size: usize) -> Self {
        Self {
            chunk: C::new(chunk_size),
            phantom: PhantomData,
        }
    }

    fn chunk_size(&self) -> usize {
        self.chunk.chunk_size()
    }

    fn pull_x(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = T>> {
        self.chunk
            .pull_x(iter.underlying_iter())
            .map(|x| x.copied())
    }
}

impl<'a, T, C> BufferedChunk<T> for CopiedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Copy,
    C: BufferedChunk<&'a T>,
{
    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        self.chunk.pull(iter.underlying_iter()).map(|x| x.copied())
    }
}
