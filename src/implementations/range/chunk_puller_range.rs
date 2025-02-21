use super::con_iter_range::ConIterRange;
use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration};
use core::ops::{Add, Range};

pub struct ChunkPullerRange<'i, T, E>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    con_iter: &'i ConIterRange<T, E>,
    chunk_size: usize,
}

impl<'i, T, E> ChunkPullerRange<'i, T, E>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    pub(super) fn new(con_iter: &'i ConIterRange<T, E>, chunk_size: usize) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<'i, T, E> ChunkPuller<E> for ChunkPullerRange<'i, T, E>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    type ChunkItem = T;

    type Iter<'c>
        = Range<T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        self.con_iter
            .progress_and_get_range(self.chunk_size)
            .map(|(begin_idx, a, b)| E::new_chunk(begin_idx, a..b))
    }
}
