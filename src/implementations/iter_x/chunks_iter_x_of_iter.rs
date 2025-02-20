use super::con_iter_x_of_iter::ConIterXOfIter;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use alloc::vec::Vec;
use core::iter::FusedIterator;
use core::ops::{Add, Range};

pub struct ChunksPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    con_iter: &'i ConIterXOfIter<I, T>,
    buffer: Vec<Option<T>>,
}

impl<'i, I, T> ChunksPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    pub(super) fn new(con_iter: &'i ConIterXOfIter<I, T>, chunk_size: usize) -> Self {
        let mut buffer = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            buffer.push(None);
        }
        Self { con_iter, buffer }
    }
}

impl<'i, I, T> ChunkPuller<Regular> for ChunksPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    type ChunkItem = T;

    type Iter = ChunksIterXOfIter<'i, T>;

    fn chunk_size(&self) -> usize {
        self.buffer.len()
    }
}

impl<'i, I, T> Iterator for ChunksPullerXOfIter<'i, I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    type Item = ChunksIterXOfIter<'i, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.con_iter.get_handle().and_then(|mut handle| {
            let iter = unsafe { &mut *self.con_iter.iter.get() };
            let mut slice_len = self.buffer.len();
            for i in 0..self.buffer.len() {
                let next = iter.next();
                match next.is_some() {
                    true => self.buffer[i] = iter.next(),
                    false => {
                        slice_len = i;
                        handle.set_target_to_completed();
                        break;
                    }
                }
            }

            match slice_len {
                0 => None,
                n => {
                    let buffer = &mut self.buffer[0..n];
                    Some(ChunksIterXOfIter { buffer, current: 0 })
                }
            }
        });
        None
    }
}

pub struct ChunksIterXOfIter<'i, T>
where
    T: Send + Sync,
{
    buffer: &'i mut [Option<T>],
    current: usize,
}

impl<'i, T> Default for ChunksIterXOfIter<'i, T>
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

impl<'i, T> Iterator for ChunksIterXOfIter<'i, T>
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

impl<'i, T> ExactSizeIterator for ChunksIterXOfIter<'i, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.current)
    }
}

impl<'i, T> FusedIterator for ChunksIterXOfIter<'i, T> where T: Send + Sync {}
