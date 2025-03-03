use super::con_iter_vec::ConIterVec;
use super::seq_chunk_iter_vec::SeqChunksIterVec;
use crate::enumeration::{Element, Enumeration};
use crate::pullers::{ChunkPuller, PulledChunkIter};

pub struct ChunkPullerVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    con_iter: &'i ConIterVec<T, E>,
    chunk_size: usize,
}

impl<'i, T, E> ChunkPullerVec<'i, T, E>
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

impl<'i, T, E> ChunkPuller<E> for ChunkPullerVec<'i, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type ChunkItem = T;

    type Iter<'c>
        = SeqChunksIterVec<'i, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|(begin_idx, first, last)| {
                E::new_chunk(begin_idx, SeqChunksIterVec::new(false, first, last))
            })
    }

    fn pulli(&mut self) -> Option<PulledChunkIter<Self::Iter<'_>, E>> {
        self.con_iter
            .progress_and_get_chunk_pointers(self.chunk_size)
            .map(|(begin_idx, first, last)| {
                let begin_idx = E::into_begin_idx(begin_idx);
                let chunk = SeqChunksIterVec::new(false, first, last);
                E::new_pulled_chunk_iter(begin_idx, chunk)
            })
    }
}
