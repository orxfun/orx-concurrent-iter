use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        default_fns,
    },
    AtomicCounter, ConcurrentIter, ExactSizeConcurrentIter, Next, NextChunk, NextManyExact,
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

    fn get(&self, item_idx: usize) -> Option<Idx> {
        let value = self.range.start + item_idx.into();
        match value.cmp(&self.range.end) {
            Ordering::Less => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> impl NextChunk<Idx> {
        self.fetch_n_with_exact_len(n)
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(Idx)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::exact_for_each(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, Idx)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::exact_for_each_with_ids(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, F: FnMut(B, Idx) -> B>(&self, chunk_size: usize, initial: B, f: F) -> B {
        default_fns::fold::exact_fold(self, chunk_size, f, initial)
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

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Idx, impl ExactSizeIterator<Item = Idx>> {
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

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(&self, chunk_size: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n(chunk_size)
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(Self::Item)>(&self, chunk_size: usize, f: F) {
        <Self as AtomicIter<_>>::for_each_n(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, Self::Item)>(&self, chunk_size: usize, f: F) {
        <Self as AtomicIter<_>>::enumerate_for_each_n(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, Fold>(&self, chunk_size: usize, neutral: B, fold: Fold) -> B
    where
        Fold: FnMut(B, Self::Item) -> B,
    {
        <Self as AtomicIter<_>>::fold(self, chunk_size, neutral, fold)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        Some(<Self as ExactSizeConcurrentIter>::len(self))
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

    fn next_exact_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        let next = <Self as AtomicIterWithInitialLen<_>>::fetch_n_with_exact_len(self, chunk_size);
        if next.is_empty() {
            None
        } else {
            Some(next)
        }
    }
}
