use super::con_iter::ConIterVec;
use crate::{implementations::array_utils::ArrayChunkSeqIter, pullers::ChunkPuller};

pub struct ChunkPullerVec<'i, T>
where
    T: Send + Sync,
{
    con_iter: &'i ConIterVec<T>,
    chunk_size: usize,
}

impl<'i, T> ChunkPullerVec<'i, T>
where
    T: Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterVec<T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, T> ChunkPuller for ChunkPullerVec<'i, T>
where
    T: Send + Sync,
{
    type ChunkItem = T;

    type Chunk<'c>
        = ArrayChunkSeqIter<'i, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|(_, first, last)| ArrayChunkSeqIter::new(first, last))
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|(begin_idx, first, last)| (begin_idx, ArrayChunkSeqIter::new(first, last)))
    }
}
