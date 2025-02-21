use super::con_iter_of_iter::ConIterOfIter;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use alloc::vec::Vec;
use core::iter::FusedIterator;
use core::marker::PhantomData;

pub struct ChunkPullerOfIter<'i, I, T, E = Regular>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    con_iter: &'i ConIterOfIter<I, T, E>,
    buffer: Vec<Option<T>>,
    phantom: PhantomData<E>,
}

impl<'i, I, T, E> ChunkPullerOfIter<'i, I, T, E>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    pub(super) fn new(con_iter: &'i ConIterOfIter<I, T, E>, chunk_size: usize) -> Self {
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

impl<'i, I, T, E> ChunkPuller<E> for ChunkPullerOfIter<'i, I, T, E>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
    E: Enumeration,
{
    type ChunkItem = T;

    type Iter<'c>
        = ChunksIterOfIter<'c, T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.buffer.len()
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        // self.con_iter
        //     .progress_and_get_chunk_pointers(self.chunk_size)
        //     .map(|(begin_idx, first, last)| {
        //         E::new_chunk(begin_idx, SeqChunksIterVec::new(false, first, last))
        //     });
        todo!()
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
