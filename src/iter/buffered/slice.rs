use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfSlice;
use std::cmp::Ordering;

pub struct BufferedSlice {
    chunk_size: usize,
}

impl BufferedSlice {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

impl<'a, T> BufferedChunk<&'a T> for BufferedSlice
where
    T: Send + Sync,
{
    type ConIter = ConIterOfSlice<'a, T>;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = &'a T>> {
        let slice = iter.as_slice();
        match begin_idx.cmp(&slice.len()) {
            Ordering::Less => {
                let end_idx = (begin_idx + self.chunk_size)
                    .min(slice.len())
                    .max(begin_idx);
                let values = slice[begin_idx..end_idx].iter();
                Some(values)
            }
            _ => None,
        }
    }
}
