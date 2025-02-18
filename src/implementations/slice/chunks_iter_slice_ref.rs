use super::con_iter_slice_ref::ConIterSliceRef;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration};

pub struct ChunksIterSliceRef<'i, 'a, T, K>
where
    T: Send + Sync,
    K: Enumeration,
{
    con_iter: &'i ConIterSliceRef<'a, T, K>,
    chunk_size: usize,
}

impl<'i, 'a, T, K> ChunksIterSliceRef<'i, 'a, T, K>
where
    T: Send + Sync,
    K: Enumeration,
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
    K: Enumeration,
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
    K: Enumeration,
{
    type Item = <K::Element as Element>::IterOf<<Self as ChunkPuller<K>>::Iter>;

    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter
            .progress_and_get_chunk(self.chunk_size)
            .map(|(begin_idx, slice)| K::new_chunk(begin_idx, slice.iter()))
    }
}
