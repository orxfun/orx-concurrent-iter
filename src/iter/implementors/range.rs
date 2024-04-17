use crate::{AtomicCounter, AtomicIter, AtomicIterWithInitialLen, NextChunk, NextManyExact};
use std::{
    cmp::Ordering,
    ops::{Add, Range, Sub},
};

/// A concurrent iterator over a slice yielding references to the elements.
#[derive(Debug, Clone)]
pub struct ConIterOfRange<Idx>
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
{
    range: Range<Idx>,
    counter: AtomicCounter,
}

impl<Idx> ConIterOfRange<Idx>
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
{
    /// Creates a concurrent iterator for the given `range`.
    pub fn new(range: Range<Idx>) -> Self {
        Self {
            range,
            counter: AtomicCounter::new(),
        }
    }
}

impl<Idx> From<Range<Idx>> for ConIterOfRange<Idx>
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
{
    /// Creates a concurrent iterator for the given `range`.
    fn from(range: Range<Idx>) -> Self {
        Self::new(range)
    }
}

impl<Idx> AtomicIter for ConIterOfRange<Idx>
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
{
    type Item = Idx;

    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    fn get(&self, item_idx: usize) -> Option<Self::Item> {
        let value = self.range.start + item_idx.into();
        match value.cmp(&self.range.end) {
            Ordering::Less => Some(value),
            _ => None,
        }
    }

    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n_with_exact_len(n)
    }
}

impl<Idx> AtomicIterWithInitialLen for ConIterOfRange<Idx>
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
{
    fn initial_len(&self) -> usize {
        (self.range.end - self.range.start).into()
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>> {
        let begin_idx = self.counter.fetch_and_add(n);

        let begin_value = self.range.start + begin_idx.into();

        let end_value = match begin_value.cmp(&self.range.end) {
            Ordering::Less => (begin_value + n.into()).min(self.range.end),
            _ => begin_value,
        };

        let end_idx: usize = (end_value - self.range.start).into();

        let values = (begin_idx..end_idx).map(Idx::from);

        NextManyExact { begin_idx, values }
    }
}

unsafe impl<Idx> Sync for ConIterOfRange<Idx> where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord
{
}

unsafe impl<Idx> Send for ConIterOfRange<Idx> where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord
{
}
