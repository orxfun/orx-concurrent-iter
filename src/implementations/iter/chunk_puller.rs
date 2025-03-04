use super::con_iter::ConIterOfIter;
use crate::pullers::ChunkPuller;
use alloc::vec::Vec;
use core::iter::FusedIterator;

pub struct ChunkPullerOfIter<'i, I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
    con_iter: &'i ConIterOfIter<I>,
    buffer: Vec<Option<I::Item>>,
}

impl<'i, I> ChunkPullerOfIter<'i, I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
    pub(super) fn new(con_iter: &'i ConIterOfIter<I>, chunk_size: usize) -> Self {
        let mut buffer = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            buffer.push(None);
        }
        Self { con_iter, buffer }
    }
}

impl<'i, I> ChunkPuller for ChunkPullerOfIter<'i, I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
    type ChunkItem = I::Item;

    type Chunk<'c>
        = ChunksIterOfIter<'c, I::Item>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.buffer.len()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        // match self.con_iter.next_chunk_to_buffer(&mut self.buffer) {
        //     (_, 0) => None,
        //     (begin_idx, slice_len) => {
        //         let buffer = &mut self.buffer[0..slice_len];
        //         let chunk_iter = ChunksIterOfIter { buffer, current: 0 };
        //         Some(E::new_chunk(begin_idx, chunk_iter))
        //     }
        // }
        None
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        match self.con_iter.next_chunk_to_buffer(&mut self.buffer) {
            (_, 0) => None,
            (begin_idx, slice_len) => {
                let buffer = &mut self.buffer[0..slice_len];
                let chunk_iter = ChunksIterOfIter { buffer, current: 0 };
                Some((begin_idx, chunk_iter))
            }
        }
    }
}

// impl<I> ChunkPuller for ChunkPullerOfIter<'_, I>
// where
//     I: Iterator,
//     I::Item: Send + Sync,
// {
//     type ChunkItem = I::Item;

//     type Iter<'c>
//         = ChunksIterOfIter<'c, I::Item>
//     where
//         Self: 'c;

//     fn chunk_size(&self) -> usize {
//         self.buffer.len()
//     }

//     fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
//         match self.con_iter.next_chunk_to_buffer(&mut self.buffer) {
//             (_, 0) => None,
//             (begin_idx, slice_len) => {
//                 let buffer = &mut self.buffer[0..slice_len];
//                 let chunk_iter = ChunksIterOfIter { buffer, current: 0 };
//                 Some(E::new_chunk(begin_idx, chunk_iter))
//             }
//         }
//     }

//     fn pulli(&mut self) -> Option<PulledChunkIter<Self::Iter<'_>, E>> {
//         match self.con_iter.next_chunk_to_buffer(&mut self.buffer) {
//             (_, 0) => None,
//             (begin_idx, slice_len) => {
//                 let buffer = &mut self.buffer[0..slice_len];
//                 let chunk = ChunksIterOfIter { buffer, current: 0 };
//                 let begin_idx = E::into_begin_idx(begin_idx);
//                 Some(E::new_pulled_chunk_iter(begin_idx, chunk))
//             }
//         }
//     }
// }

// iter

pub struct ChunksIterOfIter<'i, T>
where
    T: Send + Sync,
{
    buffer: &'i mut [Option<T>],
    current: usize,
}

impl<'i, T> Default for ChunksIterOfIter<'i, T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self {
            buffer: &mut [],
            current: 0,
        }
    }
}

impl<'i, T> Iterator for ChunksIterOfIter<'i, T>
where
    T: Send + Sync,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.get_mut(self.current).and_then(|x| {
            self.current += 1;
            x.take()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buffer.len().saturating_sub(self.current);
        (len, Some(len))
    }
}

impl<'i, T> ExactSizeIterator for ChunksIterOfIter<'i, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.current)
    }
}

impl<'i, T> FusedIterator for ChunksIterOfIter<'i, T> where T: Send + Sync {}
