use super::con_iter_slice_ref::ConIterSliceRef;
use crate::{chunks_iter::ChunksIter, next::NextKind};

pub struct ChunksIterSliceRef<'i, 'a, T, K>
where
    K: NextKind,
{
    con_iter: &'i ConIterSliceRef<'a, T, K>,
    chunk_size: usize,
}

impl<'i, 'a, T, K> ChunksIterSliceRef<'i, 'a, T, K>
where
    K: NextKind,
{
    pub(super) fn new(con_iter: &'i ConIterSliceRef<'a, T, K>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T, K> ChunksIter<K> for ChunksIterSliceRef<'i, 'a, T, K>
where
    K: NextKind,
{
    type Item = &'a T;

    type Iter = core::slice::Iter<'a, T>;

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<K::Next<Self::Iter>> {
        self.con_iter
            .progress_and_get_chunk(self.chunk_size)
            .map(|(begin_idx, slice)| K::new_next(begin_idx, slice.iter()))
    }
}
