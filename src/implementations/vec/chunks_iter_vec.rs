use super::con_iter_vec::ConIterVec;
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

impl<'i, T, E> ChunkPuller<E> for ChunksIterVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type ChunkItem = T;

    type Iter = alloc::vec::IntoIter<T>;

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
        // self.con_iter
        //     .progress_and_get_chunk(self.chunk_size)
        //     .map(|(begin_idx, slice)| K::new_chunk(begin_idx, slice.iter()))
        None
    }
}
