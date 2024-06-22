use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfVec;
use std::marker::PhantomData;

pub struct BufferedVec<T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<T> BufferedChunk<T> for BufferedVec<T>
where
    T: Send + Sync + Default,
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
        let vec = unsafe { iter.mut_vec() };
        let end_idx = (begin_idx + self.chunk_size).min(vec.len()).max(begin_idx);
        let idx_range = begin_idx..end_idx;
        let values = idx_range.map(|i| std::mem::take(&mut vec[i]));
        Some(values)
    }
}
