use super::con_iter::ConIterOfIter;
use crate::pullers::ChunkPuller;
use alloc::vec::Vec;
use core::iter::FusedIterator;

pub struct ChunkPullerOfIter<'i, I>
where
    I: Iterator,
    I::Item: Send,
{
    con_iter: &'i ConIterOfIter<I>,
    buffer: Vec<Option<I::Item>>,
}

impl<'i, I> ChunkPullerOfIter<'i, I>
where
    I: Iterator,
    I::Item: Send,
{
    pub(super) fn new(con_iter: &'i ConIterOfIter<I>, chunk_size: usize) -> Self {
        let mut buffer = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            buffer.push(None);
        }
        Self { con_iter, buffer }
    }
}

impl<I> ChunkPuller for ChunkPullerOfIter<'_, I>
where
    I: Iterator,
    I::Item: Send,
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
        match self.con_iter.next_chunk_to_buffer(&mut self.buffer) {
            (_, 0) => None,
            (_, slice_len) => {
                let buffer = &mut self.buffer[0..slice_len];
                let chunk = ChunksIterOfIter { buffer, current: 0 };
                Some(chunk)
            }
        }
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

// iter

pub struct ChunksIterOfIter<'i, T> {
    buffer: &'i mut [Option<T>],
    current: usize,
}

impl<T> Default for ChunksIterOfIter<'_, T> {
    fn default() -> Self {
        Self {
            buffer: &mut [],
            current: 0,
        }
    }
}

impl<T> Iterator for ChunksIterOfIter<'_, T> {
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

impl<T> ExactSizeIterator for ChunksIterOfIter<'_, T> {
    fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.current)
    }
}

impl<T> FusedIterator for ChunksIterOfIter<'_, T> {}
