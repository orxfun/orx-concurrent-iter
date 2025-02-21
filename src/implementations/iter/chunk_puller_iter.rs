use super::con_iter_of_iter::ConIterOfIter;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use alloc::vec::Vec;
use core::iter::FusedIterator;

pub struct ChunkPullerOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    con_iter: &'i ConIterOfIter<I, T>,
    buffer: Vec<Option<T>>,
}

// iter

pub struct ChunksIterOfIter<'i, E, T>
where
    T: Send + Sync,
    E: Enumeration,
{
    buffer: &'i mut [Option<<E::Element as Element>::ElemOf<T>>],
    current: usize,
}

impl<'i, E, T> Default for ChunksIterOfIter<'i, E, T>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn default() -> Self {
        Self {
            buffer: &mut [],
            current: 0,
        }
    }
}

impl<'i, E, T> Iterator for ChunksIterOfIter<'i, E, T>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = <E::Element as Element>::ElemOf<T>;

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

impl<'i, E, T> ExactSizeIterator for ChunksIterOfIter<'i, E, T>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.current)
    }
}

impl<'i, E, T> FusedIterator for ChunksIterOfIter<'i, E, T>
where
    T: Send + Sync,
    E: Enumeration,
{
}
