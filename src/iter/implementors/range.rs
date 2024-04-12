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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        iter::{
            atomic_iter::tests::{
                atomic_exact_fetch_n, atomic_exact_fetch_one, atomic_fetch_n, atomic_fetch_one,
                ATOMIC_FETCH_N, ATOMIC_TEST_LEN,
            },
            con_iter::tests::{test_ids_and_values, test_values},
        },
        ConcurrentIter, ConcurrentIterable,
    };
    use test_case::test_matrix;

    #[test]
    fn new() {
        let range = 3..10;
        let con_iter = ConIterOfRange::new(range);
        assert_eq!(con_iter.range, 3..10);
        assert_eq!(con_iter.counter().current(), 0);
    }

    #[test]
    fn from() {
        let range = 3..10;
        let con_iter: ConIterOfRange<_> = range.into();
        assert_eq!(con_iter.range, 3..10);
        assert_eq!(con_iter.counter().current(), 0);
    }

    #[test]
    fn debug() {
        let range = 3..10;
        let con_iter = ConIterOfRange::new(range);

        assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfRange { range: 3..10, counter: AtomicCounter { current: 0 } }"
        );

        assert_eq!(con_iter.next(), Some(3));

        assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfRange { range: 3..10, counter: AtomicCounter { current: 1 } }"
        );
    }

    #[test]
    fn clone() {
        let range = 3..6;
        let con_iter = ConIterOfRange::new(range);

        assert_eq!(con_iter.next(), Some(3));
        assert_eq!(1, con_iter.counter().current());

        let clone = con_iter.clone();
        assert_eq!(1, clone.counter().current());

        assert_eq!(clone.next(), Some(4));
        assert_eq!(clone.next(), Some(5));
        assert_eq!(3, clone.counter().current());

        assert_eq!(clone.next(), None);
        assert_eq!(4, clone.counter().current());

        assert_eq!(clone.next(), None);
        assert_eq!(5, clone.counter().current());

        assert_eq!(1, con_iter.counter().current());
    }

    #[test]
    fn atomic() {
        atomic_fetch_one(ConIterOfRange::new(0..ATOMIC_TEST_LEN));
        for n in ATOMIC_FETCH_N {
            atomic_fetch_n(ConIterOfRange::new(0..ATOMIC_TEST_LEN), n);
        }
    }

    #[test]
    fn atomic_exact() {
        atomic_exact_fetch_one(ConIterOfRange::new(0..ATOMIC_TEST_LEN));
        for n in ATOMIC_FETCH_N {
            atomic_exact_fetch_n(ConIterOfRange::new(0..ATOMIC_TEST_LEN), n);
        }
    }

    #[test_matrix(
        [1, 2, 8],
        [1, 2, 8, 64, 1025, 5483]
    )]
    fn ids_and_values(num_threads: usize, len: usize) {
        test_values(num_threads, len, (0..len).con_iter());
        test_ids_and_values(num_threads, len, (0..len).con_iter());
    }
}
