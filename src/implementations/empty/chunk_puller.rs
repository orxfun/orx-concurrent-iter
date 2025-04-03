use super::con_iter::ConIterEmpty;
use crate::pullers::ChunkPuller;

pub struct ChunkPullerEmpty<'i, T>
where
    T: Send + Sync,
{
    con_iter: &'i ConIterEmpty<T>,
    chunk_size: usize,
}

impl<'i, T> ChunkPullerEmpty<'i, T>
where
    T: Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterEmpty<T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, T> ChunkPuller for ChunkPullerEmpty<'i, T>
where
    T: Send + Sync,
{
    type ChunkItem = T;

    type Chunk<'c>
        = core::iter::Empty<T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        None
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        None
    }
}
