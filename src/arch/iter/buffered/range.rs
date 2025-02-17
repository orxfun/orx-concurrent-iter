use super::buffered_chunk::{BufferedChunk, BufferedChunkX};
use crate::{ConIterOfRange, NextChunk};
use core::ops::{Add, Range, Sub};

pub struct BufferedRange {
    chunk_size: usize,
}

impl<Idx> BufferedChunkX<Idx> for BufferedRange
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
    Range<Idx>: Iterator<Item = Idx>,
{
    type ConIter = ConIterOfRange<Idx>;

    fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull_x(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = Idx>> {
        iter.progress_and_get_begin_idx(self.chunk_size)
            .map(|begin_idx| {
                let range = iter.range();
                let begin_value = begin_idx + range.start.into();
                let end_value = (begin_value + self.chunk_size).min(range.end.into());
                (begin_value..end_value).map(Idx::from)
            })
    }
}

impl<Idx> BufferedChunk<Idx> for BufferedRange
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
    Range<Idx>: Iterator<Item = Idx>,
{
    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<Idx, impl ExactSizeIterator<Item = Idx>>> {
        iter.progress_and_get_begin_idx(self.chunk_size)
            .map(|begin_idx| {
                let range = iter.range();
                let begin_value = begin_idx + range.start.into();
                let end_value = (begin_value + self.chunk_size).min(range.end.into());
                let values = (begin_value..end_value).map(Idx::from);
                NextChunk { begin_idx, values }
            })
    }
}
