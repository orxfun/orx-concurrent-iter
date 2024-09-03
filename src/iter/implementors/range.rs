use crate::{
    iter::buffered::{
        buffered_chunk::BufferedChunk, buffered_iter::BufferedIter, range::BufferedRange,
    },
    next::NextChunk,
    AtomicCounter, ConcurrentIter, Next,
};
use std::{
    cmp::Ordering,
    ops::{Add, Range, Sub},
};

/// A concurrent iterator over a slice yielding references to the elements.
#[derive(Clone)]
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
    Range<Idx>: Iterator<Item = Idx>,
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
    Range<Idx>: Iterator<Item = Idx>,
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

    fn get(&self, item_idx: usize) -> Option<Idx> {
        let value = self.range.start + item_idx.into();
        match value.cmp(&self.range.end) {
            Ordering::Less => Some(value),
            _ => None,
        }
    }

    pub(crate) fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_and_add(number_to_fetch);
        match begin_idx.cmp(&self.initial_len()) {
            Ordering::Less => Some(begin_idx),
            _ => None,
        }
    }

    #[inline(always)]
    fn initial_len(&self) -> usize {
        let start: usize = self.range.start.into();
        let end: usize = self.range.end.into();
        end.saturating_sub(start)
    }
}

impl<Idx> std::fmt::Debug for ConIterOfRange<Idx>
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::helpers::fmt_iter(f, "ConIterOfRange", Some(self.initial_len()), &self.counter)
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
    Range<Idx>: Iterator<Item = Idx>,
{
    /// Creates a concurrent iterator for the given `range`.
    fn from(range: Range<Idx>) -> Self {
        Self::new(range)
    }
}

unsafe impl<Idx> Sync for ConIterOfRange<Idx>
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
}

unsafe impl<Idx> Send for ConIterOfRange<Idx>
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
    Range<Idx>: Iterator<Item = Idx>,
{
    type Item = Idx;

    type BufferedIter = BufferedRange;

    type SeqIter = Range<Idx>;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let range = 0..1024;
    /// let con_iter = range.con_iter();
    ///
    /// std::thread::scope(|s| {
    ///     s.spawn(|| {
    ///         for _ in 0..42 {
    ///             _ = con_iter.next();
    ///         }
    ///
    ///         let mut buffered = con_iter.buffered_iter(32);
    ///         let _chunk = buffered.next().unwrap();
    ///     });
    /// });
    ///
    /// let num_used = 42 + 32;
    ///
    /// // converts the remaining elements into a sequential iterator
    /// let seq_iter = con_iter.into_seq_iter();
    ///
    /// assert_eq!(seq_iter.len(), 1024 - num_used);
    /// for (i, x) in seq_iter.enumerate() {
    ///     assert_eq!(x, num_used + i);
    /// }
    /// ```
    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.current();
        (self.range.start + current.into())..self.range.end
    }

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        let idx = self.counter.fetch_and_increment();
        self.get(idx).map(|value| Next { idx, value })
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(chunk_size)
            .unwrap_or(self.initial_len());
        let begin_value = begin_idx + self.range.start.into();
        let end_value = match begin_value.cmp(&self.range.end.into()) {
            Ordering::Less => (begin_value + chunk_size).min(self.range.end.into()),
            _ => begin_value,
        };
        let end_idx: usize = end_value - self.range.start.into();

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let values = (begin_value..end_value).map(Idx::from);
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        let current = self.counter.current();
        let initial_len = self.initial_len();
        let len = match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        };
        Some(len)
    }

    fn skip_to_end(&self) {
        let _ = self.counter.get_current_max_value(self.range.end.into());
    }
}
