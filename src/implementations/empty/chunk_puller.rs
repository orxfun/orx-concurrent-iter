use super::con_iter::ConIterEmpty;
use crate::pullers::ChunkPuller;
use core::marker::PhantomData;

pub struct ChunkPullerEmpty<'i, T>
where
    T: Send + Sync,
{
    chunk_size: usize,
    phantom: PhantomData<&'i T>,
}

impl<'i, T> ChunkPullerEmpty<'i, T>
where
    T: Send + Sync,
{
    pub(super) fn new(_con_iter: &'i ConIterEmpty<T>, chunk_size: usize) -> Self {
        Self {
            chunk_size,
            phantom: PhantomData,
        }
    }
}

impl<T> ChunkPuller for ChunkPullerEmpty<'_, T>
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
