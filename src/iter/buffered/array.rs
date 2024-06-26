use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfArray;
use std::marker::PhantomData;

pub struct BufferedArray<const N: usize, T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<const N: usize, T> BufferedChunk<T> for BufferedArray<N, T>
where
    T: Send + Sync,
{
    type ConIter = ConIterOfArray<N, T>;

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
