use super::buffered_chunk::BufferedChunk;
use crate::ClonedConIterOfSlice;
use std::{cmp::Ordering, marker::PhantomData};

pub struct BufferedSliceCloned<'a> {
    chunk_size: usize,
    phantom: PhantomData<&'a ()>,
}

impl<'a> BufferedSliceCloned<'a> {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            phantom: Default::default(),
        }
    }
}

impl<'a, T> BufferedChunk<T> for BufferedSliceCloned<'a>
where
    T: Send + Sync + Clone + 'a,
{
    type ConIter = ClonedConIterOfSlice<'a, T>;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = T>> {
        let slice = iter.as_slice();
        match begin_idx.cmp(&slice.len()) {
            Ordering::Less => {
                let end_idx = (begin_idx + self.chunk_size)
                    .min(slice.len())
                    .max(begin_idx);
                let values = slice[begin_idx..end_idx].iter().cloned();
                Some(values)
            }
            _ => None,
        }
    }
}
