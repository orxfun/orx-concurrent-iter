use super::{con_iter::ConIterJaggedRef, slice_iter::RawJaggedSliceIterRef};
use crate::{
    ChunkPuller,
    implementations::jagged_arrays::{JaggedIndexer, Slices},
};

pub struct ChunkPullerJaggedRef<'i, 'a, T, S, X>
where
    T: Sync,
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    con_iter: &'i ConIterJaggedRef<'a, T, S, X>,
    chunk_size: usize,
}

impl<'i, 'a, T, S, X> ChunkPullerJaggedRef<'i, 'a, T, S, X>
where
    T: Sync,
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    pub(super) fn new(con_iter: &'i ConIterJaggedRef<'a, T, S, X>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'a, T, S, X> ChunkPuller for ChunkPullerJaggedRef<'_, 'a, T, S, X>
where
    T: Sync,
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    type ChunkItem = &'a T;

    type Chunk<'c>
        = RawJaggedSliceIterRef<'a, T, S, X>
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
        self.con_iter.progress_and_get_iter(self.chunk_size)
    }
}
