use super::buffered_chunk::{BufferedChunk, BufferedChunkX};
use crate::{iter::implementors::iter::ConIterOfIter, NextChunk};
use alloc::vec::Vec;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct BufferIter<T, Iter>
where
    T: Send + Sync,
    Iter: Iterator<Item = T>,
{
    values: Vec<Option<T>>,
    phantom: PhantomData<Iter>,
}

impl<T, Iter> BufferedChunkX<T> for BufferIter<T, Iter>
where
    T: Send + Sync,
    Iter: Iterator<Item = T>,
{
    type ConIter = ConIterOfIter<T, Iter>;

    fn new(chunk_size: usize) -> Self {
        Self {
            values: (0..chunk_size).map(|_| None).collect(),
            phantom: PhantomData,
        }
    }

    fn chunk_size(&self) -> usize {
        self.values.len()
    }

    fn pull_x(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = T>> {
        iter.progress_and_get_begin_idx(self.chunk_size())
            .and_then(|begin_idx| {
                // SAFETY: no other thread has the valid condition to iterate, they are waiting

                let core_iter = unsafe { &mut *iter.iter.get() };

                let mut count = 0;
                for pos in self.values.iter_mut() {
                    match core_iter.next() {
                        Some(x) => {
                            *pos = Some(x);
                            count += 1;
                        }
                        None => break,
                    }
                }

                match count == self.chunk_size() {
                    true => iter.release_handle(begin_idx + self.chunk_size()),
                    false => iter.release_handle_complete(),
                }

                match count {
                    0 => None,
                    _ => Some(BufferedIter {
                        values: &mut self.values,
                        initial_len: count,
                        current_idx: 0,
                    }),
                }
            })
    }
}

impl<T, Iter> BufferedChunk<T> for BufferIter<T, Iter>
where
    T: Send + Sync,
    Iter: Iterator<Item = T>,
{
    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        iter.progress_and_get_begin_idx(self.chunk_size())
            .and_then(|begin_idx| {
                // SAFETY: no other thread has the valid condition to iterate, they are waiting

                let core_iter = unsafe { &mut *iter.iter.get() };

                let mut count = 0;
                for pos in self.values.iter_mut() {
                    match core_iter.next() {
                        Some(x) => {
                            *pos = Some(x);
                            count += 1;
                        }
                        None => break,
                    }
                }

                match count == self.chunk_size() {
                    true => iter.release_handle(begin_idx + self.chunk_size()),
                    false => iter.release_handle_complete(),
                }

                match count {
                    0 => None,
                    _ => {
                        let values = BufferedIter {
                            values: &mut self.values,
                            initial_len: count,
                            current_idx: 0,
                        };
                        Some(NextChunk { begin_idx, values })
                    }
                }
            })
    }
}

// Iterator
pub struct BufferedIter<'a, T> {
    values: &'a mut [Option<T>],
    initial_len: usize,
    current_idx: usize,
}

impl<T> Iterator for BufferedIter<'_, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx < self.initial_len {
            let next = self.values[self.current_idx].take();
            if next.is_some() {
                self.current_idx += 1;
            } else {
                self.current_idx = self.initial_len;
            }
            next
        } else {
            None
        }
    }
}

impl<T> ExactSizeIterator for BufferedIter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.initial_len - self.current_idx
    }
}
