use super::buffered_chunk::{BufferedChunk, BufferedChunkX};
use crate::{ConIterOfSlice, NextChunk};
use std::marker::PhantomData;

pub struct BufferedSlice<T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<'a, T> BufferedChunkX<&'a T> for BufferedSlice<T>
where
    T: Send + Sync,
{
    type ConIter = ConIterOfSlice<'a, T>;

    fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            phantom: PhantomData,
        }
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull_x(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = &'a T>> {
        iter.progress_and_get_begin_idx(self.chunk_size)
            .map(|begin_idx| {
                let slice = iter.as_slice();
                let end_idx = (begin_idx + self.chunk_size)
                    .min(slice.len())
                    .max(begin_idx);
                slice[begin_idx..end_idx].iter()
            })
    }
}

impl<'a, T> BufferedChunk<&'a T> for BufferedSlice<T>
where
    T: Send + Sync,
{
    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<&'a T, impl ExactSizeIterator<Item = &'a T>>> {
        iter.progress_and_get_begin_idx(self.chunk_size)
            .map(|begin_idx| {
                let slice = iter.as_slice();
                let end_idx = (begin_idx + self.chunk_size)
                    .min(slice.len())
                    .max(begin_idx);
                let values = slice[begin_idx..end_idx].iter();
                NextChunk { begin_idx, values }
            })
    }
}
