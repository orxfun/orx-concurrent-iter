use super::{
    as_raw_jagged_ref::AsRawJaggedRef, con_iter::ConIterJaggedRef,
    slice_iter::RawJaggedSliceIterRef,
};
use crate::{ChunkPuller, implementations::jagged_arrays::JaggedIndexer};

pub struct ChunkPullerJaggedRef<'i, 'a, J, X, T>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer + Send + Sync + 'a,
    J: AsRawJaggedRef<'a, T, X>,
{
    con_iter: &'i ConIterJaggedRef<'a, J, X, T>,
    chunk_size: usize,
}

impl<'i, 'a, J, X, T> ChunkPullerJaggedRef<'i, 'a, J, X, T>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer + Send + Sync + 'a,
    J: AsRawJaggedRef<'a, T, X>,
{
    pub(super) fn new(con_iter: &'i ConIterJaggedRef<'a, J, X, T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, J, X, T> ChunkPuller for ChunkPullerJaggedRef<'i, 'a, J, X, T>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer + Send + Sync + 'a,
    J: AsRawJaggedRef<'a, T, X> + 'a,
{
    type ChunkItem = &'a T;

    type Chunk<'c>
        = RawJaggedSliceIterRef<'a, J, X, T>
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
