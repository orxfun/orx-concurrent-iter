use super::con_iter_slice::ConIterSliceRef;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration};

pub struct ChunksIterSliceRef<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    con_iter: &'i ConIterSliceRef<'a, T, E>,
    chunk_size: usize,
}

impl<'i, 'a, T, E> ChunksIterSliceRef<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(super) fn new(con_iter: &'i ConIterSliceRef<'a, T, E>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T, E> ChunkPuller<E> for ChunksIterSliceRef<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type ChunkItem = &'a T;

    type Iter = core::slice::Iter<'a, T>;

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl<'i, 'a, T, E> Iterator for ChunksIterSliceRef<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = <E::Element as Element>::IterOf<<Self as ChunkPuller<E>>::Iter>;

    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter
            .progress_and_get_slice(self.chunk_size)
            .map(|(begin_idx, slice)| E::new_chunk(begin_idx, slice.iter()))
    }
}
