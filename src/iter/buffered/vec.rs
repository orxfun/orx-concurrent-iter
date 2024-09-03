use super::buffered_chunk::BufferedChunk;
use crate::{ConIterOfVec, NextChunk};
use std::marker::PhantomData;

pub struct BufferedVec<T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<T> BufferedChunk<T> for BufferedVec<T>
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
