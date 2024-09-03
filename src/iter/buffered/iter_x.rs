use super::buffered_chunk::BufferedChunkX;
use crate::iter::implementors::iter_x::ConIterOfIterX;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BufferIterX<T, Iter>
where
    T: Send + Sync,
    Iter: Iterator<Item = T>,
{
    values: Vec<Option<T>>,
    phantom: PhantomData<Iter>,
}

impl<T, Iter> BufferedChunkX<T> for BufferIterX<T, Iter>
where
    T: Send + Sync,
    Iter: Iterator<Item = T>,
{
    type ConIter = ConIterOfIterX<T, Iter>;

    fn new(chunk_size: usize) -> Self {
        Self {
            values: (0..chunk_size).map(|_| None).collect(),
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn chunk_size(&self) -> usize {
        self.values.len()
    }

    fn pull(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = T>> {
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
            true => iter.release_handle(),
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
    }
}

// Iterator
pub struct BufferedIter<'a, T> {
    values: &'a mut [Option<T>],
    initial_len: usize,
    current_idx: usize,
}

impl<'a, T> Iterator for BufferedIter<'a, T> {
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

impl<'a, T> ExactSizeIterator for BufferedIter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.initial_len - self.current_idx
    }
}
