use super::buffered_chunk::BufferedChunk;
use crate::ConIterOfRange;
use std::{
    cmp::Ordering,
    ops::{Add, Range, Sub},
};

pub struct BufferedRange {
    chunk_size: usize,
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
    type ConIter = ConIterOfRange<Idx>;

    fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = Idx>> {
        let range = iter.range();
        let begin_value = begin_idx + range.start.into();
        match begin_value.cmp(&range.end.into()) {
            Ordering::Less => {
                let end_value = (begin_value + self.chunk_size).min(range.end.into());
                let values = (begin_value..end_value).map(Idx::from);
                Some(values)
            }
            _ => None,
        }
    }
}
