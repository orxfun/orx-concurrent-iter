use super::con_iter_vec::ConIterVec;
use super::seq_chunk_iter_vec::SeqChunksIterVec;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration};

pub struct ChunksIterVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    con_iter: &'i ConIterVec<T, E>,
    chunk_size: usize,
}

impl<'i, T, E> ChunksIterVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(super) fn new(con_iter: &'i ConIterVec<T, E>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, T, E> ChunkPuller<E> for ChunksIterVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type ChunkItem = T;

    type Iter = SeqChunksIterVec<'i, T>;

    fn chunk_size(&self) -> usize {
        todo!()
    }
}

impl<'i, T, E> Iterator for ChunksIterVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = <E::Element as Element>::IterOf<<Self as ChunkPuller<E>>::Iter>;

    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|(begin_idx, first, last)| {
                E::new_chunk(begin_idx, SeqChunksIterVec::new(false, first, last))
            })
    }
}
