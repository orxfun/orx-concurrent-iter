use super::con_iter::ConIterSlice;
use crate::pullers::ChunkPuller;

pub struct ChunkPullerSlice<'i, 'a, T> {
    con_iter: &'i ConIterSlice<'a, T>,
    chunk_size: usize,
}

impl<'i, 'a, T> ChunkPullerSlice<'i, 'a, T> {
    pub(super) fn new(con_iter: &'i ConIterSlice<'a, T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'a, T> ChunkPuller for ChunkPullerSlice<'_, 'a, T> {
    type ChunkItem = &'a T;

    type Chunk<'c>
        = core::slice::Iter<'a, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.con_iter
            .progress_and_get_slice(self.chunk_size)
            .map(|(_, slice)| slice.iter())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.con_iter
            .progress_and_get_slice(self.chunk_size)
            .map(|(begin_idx, slice)| (begin_idx, slice.iter()))
    }
}
