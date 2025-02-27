use super::con_iter_of_iter::ConIterOfIter;
use crate::pullers::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use alloc::vec::Vec;
use core::iter::FusedIterator;
use core::marker::PhantomData;

pub struct ChunkPullerOfIter<'i, I, E = Regular>
where
    I: Iterator,
    I::Item: Send + Sync,
    E: Enumeration,
{
    con_iter: &'i ConIterOfIter<I, E>,
    buffer: Vec<Option<I::Item>>,
    phantom: PhantomData<E>,
}

impl<'i, I, E> ChunkPullerOfIter<'i, I, E>
where
    I: Iterator,
    I::Item: Send + Sync,
    E: Enumeration,
{
    pub(super) fn new(con_iter: &'i ConIterOfIter<I, E>, chunk_size: usize) -> Self {
        let mut buffer = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            buffer.push(None);
        }
        Self {
            con_iter,
            buffer,
            phantom: PhantomData,
        }
    }
}

impl<'i, I, E> ChunkPuller<E> for ChunkPullerOfIter<'i, I, E>
where
    I: Iterator,
    I::Item: Send + Sync,
    E: Enumeration,
{
    type ChunkItem = I::Item;

    type Iter<'c>
        = ChunksIterOfIter<'c, I::Item>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.buffer.len()
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        match self.con_iter.next_chunk_to_buffer(&mut self.buffer) {
            (_, 0) => None,
            (begin_idx, slice_len) => {
                let buffer = &mut self.buffer[0..slice_len];
                let chunk_iter = ChunksIterOfIter { buffer, current: 0 };
                Some(E::new_chunk(begin_idx, chunk_iter))
            }
        }
    }
}

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
