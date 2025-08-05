use super::chunk_seq_iter::ArrayChunkSeqIter;
use super::con_iter::ArrayConIter;
use crate::pullers::ChunkPuller;

pub struct ArrayChunkPuller<'i, C>
where
    C: ArrayConIter,
{
    con_iter: &'i C,
    chunk_size: usize,
}

impl<'i, C> ArrayChunkPuller<'i, C>
where
    C: ArrayConIter,
{
    pub(crate) fn new(con_iter: &'i C, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, C> ChunkPuller for ArrayChunkPuller<'i, C>
where
    C: ArrayConIter,
{
    type ChunkItem = C::Item;

    type Chunk<'c>
        = ArrayChunkSeqIter<'i, C::Item>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|x| ArrayChunkSeqIter::new(x.first, x.last))
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|x| (x.begin_idx, ArrayChunkSeqIter::new(x.first, x.last)))
    }
}
