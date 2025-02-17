use super::con_iter_slice_ref::ConIterSliceRef;
use crate::{chunks_iter::ChunksIter, next::Next};

pub struct ChunksIterSliceRef<'i, 'a, T> {
    con_iter: &'i ConIterSliceRef<'a, T>,
    chunk_size: usize,
}

impl<'i, 'a, T> ChunksIterSliceRef<'i, 'a, T> {
    pub(super) fn new(con_iter: &'i ConIterSliceRef<'a, T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T> ChunksIter for ChunksIterSliceRef<'i, 'a, T> {
    type Item = &'a T;

    type Iter = core::slice::Iter<'a, T>;

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull<N: Next<Self::Iter>>(&mut self) -> Option<N> {
        self.con_iter
            .progress_and_get_chunk(self.chunk_size)
            .map(|(begin_idx, slice)| N::new(begin_idx, slice.iter()))
    }
}
