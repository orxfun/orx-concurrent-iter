use super::con_iter::ConIterSliceMut;
use crate::pullers::ChunkPuller;

pub struct ChunkPullerSliceMut<'i, 'a, T> {
    con_iter: &'i ConIterSliceMut<'a, T>,
    chunk_size: usize,
}

impl<'i, 'a, T> ChunkPullerSliceMut<'i, 'a, T> {
    pub(super) fn new(con_iter: &'i ConIterSliceMut<'a, T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'a, T> ChunkPuller for ChunkPullerSliceMut<'_, 'a, T> {
    type ChunkItem = &'a mut T;

    type Chunk<'c>
        = core::slice::IterMut<'a, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        let slice = unsafe { self.con_iter.progress_and_get_slice(self.chunk_size) };
        slice.map(|(_, slice)| slice.iter_mut())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        let slice = unsafe { self.con_iter.progress_and_get_slice(self.chunk_size) };
        slice.map(|(begin_idx, slice)| (begin_idx, slice.iter_mut()))
    }
}
