use super::con_iter_ref::ConIterJaggedRef;
use crate::{
    ChunkPuller,
    implementations::jagged::{iter::RawJaggedSliceIterRef, jagged_indexer::JaggedIndexer},
};

pub struct ChunkPullerJaggedRef<'i, 'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
    con_iter: &'i ConIterJaggedRef<'a, T, X>,
    chunk_size: usize,
}

impl<'i, 'a, T, X> ChunkPullerJaggedRef<'i, 'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterJaggedRef<'a, T, X>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T, X> ChunkPuller for ChunkPullerJaggedRef<'i, 'a, T, X>
where
    T: Send + Sync,
    X: JaggedIndexer + Send + Sync,
{
    type ChunkItem = &'a T;

    type Chunk<'c>
        = RawJaggedSliceIterRef<'a, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.con_iter
            .progress_and_get_iter(self.chunk_size)
            .map(|(_, iter)| iter)
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.con_iter
            .progress_and_get_iter(self.chunk_size)
            .map(|(begin_idx, iter)| (begin_idx, iter))
    }
}
