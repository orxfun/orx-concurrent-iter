use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfVec;
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
            phantom: Default::default(),
        }
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = T>> {
        Some(unsafe { iter.take_slice(begin_idx, self.chunk_size) })
    }
}
