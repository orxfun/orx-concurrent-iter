use super::con_iter_range::ConIterRange;
use crate::enumeration::{Element, Enumeration};
use crate::pullers::{ChunkPuller, PulledChunkIter};
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

impl<T, E> ChunkPuller<E> for ChunkPullerRange<'_, T, E>
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

    fn pulli(&mut self) -> Option<PulledChunkIter<Self::Iter<'_>, E>> {
        self.con_iter
            .progress_and_get_range(self.chunk_size)
            .map(|(begin_idx, a, b)| {
                let begin_idx = E::into_begin_idx(begin_idx);
                let chunk = a..b;
                E::new_pulled_chunk_iter(begin_idx, chunk)
            })
    }
}
