use crate::ChunkPuller;

use super::con_iter_owned::ConIterJaggedOwned;

pub struct ChunkPullerJaggedOwned<'i, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
    con_iter: &'i ConIterJaggedOwned<T, X>,
    chunk_size: usize,
}

impl<'i, T, X> ChunkPullerJaggedOwned<'i, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterJaggedOwned<T, X>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, T, X> ChunkPuller for ChunkPullerJaggedOwned<'i, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Send + Sync,
{
    type ChunkItem = T;

    type Chunk<'c>
        = core::iter::Empty<T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        todo!()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        todo!()
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        todo!()
    }
}
