use super::{con_iter::ConIterJaggedOwned, slice_iter::RawJaggedSliceIterOwned};
use crate::{
    ChunkPuller,
    implementations::jagged_arrays::{as_slice::AsOwningSlice, indexer::JaggedIndexer},
};

pub struct ChunkPullerJaggedOwned<'i, S, T, X>
where
    T: Send + Sync,
    S: AsOwningSlice<T>,
    X: JaggedIndexer + Send + Sync,
{
    con_iter: &'i ConIterJaggedOwned<S, T, X>,
    chunk_size: usize,
}

impl<'i, S, T, X> ChunkPullerJaggedOwned<'i, S, T, X>
where
    T: Send + Sync,
    S: AsOwningSlice<T>,
    X: JaggedIndexer + Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterJaggedOwned<S, T, X>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, S, T, X> ChunkPuller for ChunkPullerJaggedOwned<'i, S, T, X>
where
    T: Send + Sync,
    S: AsOwningSlice<T>,
    X: JaggedIndexer + Send + Sync,
{
    type ChunkItem = T;

    type Chunk<'c>
        = RawJaggedSliceIterOwned<'c, S, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.con_iter
            .progress_and_get_iter(self.chunk_size)
            .map(|(_begin_idx, iter)| iter)
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.con_iter.progress_and_get_iter(self.chunk_size)
    }
}
