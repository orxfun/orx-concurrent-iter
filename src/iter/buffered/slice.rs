use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfSlice;
use std::{cmp::Ordering, marker::PhantomData};

pub struct BufferedSlice<T> {
    chunk_size: usize,
    phantom: PhantomData<T>,
}

impl<'a, T> BufferedChunk<&'a T> for BufferedSlice<T>
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
