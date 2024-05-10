use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        buffered::{buffered_iter::BufferedIter, range::BufferedRange},
    },
    next::NextChunk,
    AtomicCounter, ConcurrentIter, ExactSizeConcurrentIter, Next,
};
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

    pub(crate) fn range(&self) -> &Range<Idx> {
        &self.range
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

impl<Idx> AtomicIter<Idx> for ConIterOfRange<Idx>
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
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    #[inline(always)]
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter().fetch_and_add(number_to_fetch);
        match begin_idx.cmp(&self.initial_len()) {
            Ordering::Less => Some(begin_idx),
            _ => None,
        }
    }

    fn get(&self, item_idx: usize) -> Option<Idx> {
        let value = self.range.start + item_idx.into();
        match value.cmp(&self.range.end) {
            Ordering::Less => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> Option<NextChunk<Idx, impl ExactSizeIterator<Item = Idx>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(n)
            .unwrap_or(self.initial_len());

        let begin_value = self.range.start + begin_idx.into();

        let end_value = match begin_value.cmp(&self.range.end) {
            Ordering::Less => (begin_value + n.into()).min(self.range.end),
            _ => begin_value,
        };

        let end_idx: usize = (end_value - self.range.start).into();

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let values = (begin_idx..end_idx).map(Idx::from);
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn early_exit(&self) {
        self.counter().store(self.range.end.into())
    }
}

impl<Idx> AtomicIterWithInitialLen<Idx> for ConIterOfRange<Idx>
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
    #[inline(always)]
    fn initial_len(&self) -> usize {
        (self.range.end - self.range.start).into()
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

// AtomicIter -> ConcurrentIter

impl<Idx> ConcurrentIter for ConIterOfRange<Idx>
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

    type BufferedIter = BufferedRange;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        self.fetch_n(chunk_size)
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        Some(<Self as ExactSizeConcurrentIter>::len(self))
    }

    fn skip_to_end(&self) {
        self.early_exit()
    }
}

impl<Idx> ExactSizeConcurrentIter for ConIterOfRange<Idx>
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
    #[inline(always)]
    fn len(&self) -> usize {
        let current = <Self as AtomicIter<_>>::counter(self).current();
        let initial_len = <Self as AtomicIterWithInitialLen<_>>::initial_len(self);
        match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        }
    }
}
