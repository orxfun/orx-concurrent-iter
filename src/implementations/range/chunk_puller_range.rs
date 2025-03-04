use super::con_iter_range::ConIterRange;
use crate::chunk_puller::ChunkPuller;
use core::ops::Range;

pub struct ChunkPullerRange<'i, T> {
    con_iter: &'i ConIterRange<T>,
    chunk_size: usize,
}

impl<'i, T> From<(&'i ConIterRange<T>, usize)> for ChunkPullerRange<'i, T>
where
    T: Send + Sync + From<usize>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    fn from((con_iter, chunk_size): (&'i ConIterRange<T>, usize)) -> Self {
        Self {
            con_iter,
            chunk_size,
        }
    }
}

impl<T> ChunkPuller for ChunkPullerRange<'_, T>
where
    T: Send + Sync + From<usize>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    type ChunkItem = T;

    type Chunk = Range<T>;

    fn pull(&mut self) -> Option<Self::Chunk> {
        self.con_iter
            .progress_and_get_range(self.chunk_size)
            .map(|(_, a, b)| a..b)
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)> {
        self.con_iter
            .progress_and_get_range(self.chunk_size)
            .map(|(begin_idx, a, b)| (begin_idx, a..b))
    }
}
