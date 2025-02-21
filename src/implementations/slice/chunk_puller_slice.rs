use super::con_iter_slice::ConIterSlice;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration};

pub struct ChunkPullerSlice<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    con_iter: &'i ConIterSlice<'a, T, E>,
    chunk_size: usize,
}

impl<'i, 'a, T, E> ChunkPullerSlice<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(super) fn new(con_iter: &'i ConIterSlice<'a, T, E>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T, E> ChunkPuller<E> for ChunkPullerSlice<'i, 'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type ChunkItem = &'a T;

    type Iter<'c>
        = core::slice::Iter<'a, T>
    where
        Self: 'c;

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        self.con_iter
            .progress_and_get_slice(self.chunk_size)
            .map(|(begin_idx, slice)| E::new_chunk(begin_idx, slice.iter()))
    }
}
