use super::con_iter_slice_ref::ConIterSliceRef;
use crate::{chunk_puller::ChunkPuller, next::NextKind};

pub struct ChunksIterSliceRef<'i, 'a, T, K>
where
    T: Send + Sync,
    K: NextKind,
{
    con_iter: &'i ConIterSliceRef<'a, T, K>,
    chunk_size: usize,
}

impl<'i, 'a, T, K> ChunksIterSliceRef<'i, 'a, T, K>
where
    T: Send + Sync,
    K: NextKind,
{
    pub(super) fn new(con_iter: &'i ConIterSliceRef<'a, T, K>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T, K> ChunkPuller<K> for ChunksIterSliceRef<'i, 'a, T, K>
where
    T: Send + Sync,
    K: NextKind,
{
    type ChunkItem = &'a T;

    type Iter = core::slice::Iter<'a, T>;

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl<'i, 'a, T, K> Iterator for ChunksIterSliceRef<'i, 'a, T, K>
where
    T: Send + Sync,
    K: NextKind,
{
    type Item = K::NextChunk<<Self as ChunkPuller<K>>::ChunkItem, <Self as ChunkPuller<K>>::Iter>;

    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter
            .progress_and_get_chunk(self.chunk_size)
            .map(|(begin_idx, slice)| K::new_chunk(begin_idx, slice.iter()))
    }
}
