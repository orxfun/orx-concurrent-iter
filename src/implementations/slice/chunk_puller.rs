use crate::pullers::ChunkPuller;

use super::con_iter::ConIterSlice;

pub struct ChunkPullerSlice<'i, 'a, T>
where
    T: Send + Sync,
{
    con_iter: &'i ConIterSlice<'a, T>,
    chunk_size: usize,
}

impl<'i, 'a, T> ChunkPullerSlice<'i, 'a, T>
where
    T: Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterSlice<'a, T>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, 'a, T> ChunkPuller for ChunkPullerSlice<'i, 'a, T>
where
    T: Send + Sync,
{
    type ChunkItem = &'a T;

    type Chunk<'c>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        todo!()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        todo!()
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        todo!()
    }
}

// impl<'a, T, E> ChunkPuller<E> for ChunkPullerSlice<'_, 'a, T, E>
// where
//     T: Send + Sync,
// {
//     type ChunkItem = &'a T;

//     type Iter<'c>
//         = core::slice::Iter<'a, T>
//     where
//         Self: 'c;

//     #[inline(always)]
//     fn chunk_size(&self) -> usize {
//         self.chunk_size
//     }

//     fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
//         self.con_iter
//             .progress_and_get_slice(self.chunk_size)
//             .map(|(begin_idx, slice)| E::new_chunk(begin_idx, slice.iter()))
//     }

//     fn pulli(&mut self) -> Option<PulledChunkIter<Self::Iter<'_>, E>> {
//         self.con_iter
//             .progress_and_get_slice(self.chunk_size)
//             .map(|(begin_idx, slice)| {
//                 let begin_idx = E::into_begin_idx(begin_idx);
//                 let chunk = slice.iter();
//                 E::new_pulled_chunk_iter(begin_idx, chunk)
//             })
//     }
// }
