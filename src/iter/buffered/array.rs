use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfArray;
use std::marker::PhantomData;

pub struct BufferedArray<const N: usize, T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<const N: usize, T> BufferedChunk<T> for BufferedArray<N, T>
where
    T: Send + Sync + Default,
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
        let array = unsafe { iter.mut_array() };
        let end_idx = (begin_idx + self.chunk_size)
            .min(array.len())
            .max(begin_idx);
        let idx_range = begin_idx..end_idx;
        let values = idx_range.map(|i| std::mem::take(&mut array[i]));
        Some(values)
    }
}
