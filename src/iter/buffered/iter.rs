use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfIter;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BufferIter<T, Iter>
where
    T: Send + Sync,
    Iter: Iterator<Item = T>,
{
    values: Vec<Option<T>>,
    phantom: PhantomData<Iter>,
}

impl<T, Iter> BufferedChunk<T> for BufferIter<T, Iter>
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

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = T>> {
        iter.mut_handle().and_then(|_h| {
            let core_iter = unsafe { &mut *iter.iter.get() };

            let mut i = 0;
            loop {
                match core_iter.next() {
                    Some(x) => self.values[i] = Some(x),
                    None => break,
                }

                i += 1;
                if i == self.chunk_size() {
                    break;
                }
            }

            let older_count = iter.progress_yielded_counter(self.chunk_size());
            assert_eq!(older_count, begin_idx);

            match i {
                0 => None,
                _ => {
                    let iter = BufferedIter {
                        values: &mut self.values,
                        initial_len: i,
                        current_idx: 0,
                    };
                    Some(iter)
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
