use super::buffered_chunk::{BufferedChunk, BufferedChunkX};
use crate::{ConIterOfVec, NextChunk};
use core::marker::PhantomData;

pub struct BufferedVec<T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<T> BufferedChunkX<T> for BufferedVec<T>
where
    T: Send + Sync,
{
    type ConIter = ConIterOfVec<T>;

    fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            phantom: PhantomData,
        }
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull_x(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = T>> {
        iter.progress_and_get_begin_idx(self.chunk_size)
            .map(|begin_idx| unsafe { iter.take_slice(begin_idx, self.chunk_size) })
    }
}

impl<T> BufferedChunk<T> for BufferedVec<T>
where
    T: Send + Sync,
{
    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        iter.progress_and_get_begin_idx(self.chunk_size)
            .map(|begin_idx| {
                let values = unsafe { iter.take_slice(begin_idx, self.chunk_size) };
                NextChunk { begin_idx, values }
            })
    }
}
