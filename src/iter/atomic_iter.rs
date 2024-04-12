use super::{
    atomic_counter::AtomicCounter,
    con_iter::{ConcurrentIter, ExactSizeConcurrentIter},
};
use crate::{next::Next, NextChunk, NextManyExact};
use std::cmp::Ordering;

/// Trait defining a set of concurrent iterators which internally uses an atomic counter of elements to be yielded.
///
/// Note that every `A: AtomicIter` also implements `ConcurrentIter`.
pub trait AtomicIter: Send + Sync {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Returns a reference to the underlying advanced item counter.
    fn counter(&self) -> &AtomicCounter;

    /// Returns the `item_idx`-th item that the iterator yields; returns None if the iterator completes before.
    fn get(&self, item_idx: usize) -> Option<Self::Item>;

    /// Returns the next item that the iterator yields; returns None if the iteration has completed.
    #[inline(always)]
    fn fetch_one(&self) -> Option<Next<Self::Item>> {
        let idx = self.counter().fetch_and_increment();
        self.get(idx).map(|value| Next { idx, value })
    }

    /// Returns an iterator of the next `n` **consecutive items** that the iterator yields.
    /// It might return an iterator of less or no items if the iteration does not have sufficient elements left.
    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item>;
}

/// An atomic counter based iterator with exactly known initial length.
pub trait AtomicIterWithInitialLen: AtomicIter {
    /// Returns the initial length of the atomic iterator.
    fn initial_len(&self) -> usize;

    /// Returns an iterator of the next `n` **consecutive items** that the iterator together with an exact size iterator.
    /// It might return an iterator of less or no items if the iteration does not have sufficient elements left.
    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>>;
}

impl<A: AtomicIter> ConcurrentIter for A {
    type Item = A::Item;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<<Self as AtomicIter>::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(&self, n: usize) -> impl NextChunk<<Self as AtomicIter>::Item> {
        self.fetch_n(n)
    }
}

impl<A: AtomicIterWithInitialLen> ExactSizeConcurrentIter for A {
    fn len(&self) -> usize {
        let current = self.counter().current();
        match current.cmp(&self.initial_len()) {
            Ordering::Less => self.initial_len() - current,
            _ => 0,
        }
    }

    fn next_exact_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        let next = self.fetch_n_with_exact_len(chunk_size);
        if next.is_empty() {
            None
        } else {
            Some(next)
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::ops::Add;

    pub(crate) const ATOMIC_TEST_LEN: usize = 512;

    pub(crate) const ATOMIC_FETCH_N: [usize; 8] = [
        1,
        2,
        4,
        8,
        ATOMIC_TEST_LEN / 2,
        ATOMIC_TEST_LEN,
        ATOMIC_TEST_LEN + 1,
        ATOMIC_TEST_LEN * 2,
    ];

    pub(crate) fn atomic_fetch_one<A>(iter: A)
    where
        A: AtomicIter,
        A::Item: Add<usize, Output = usize>,
    {
        assert_eq!(0, iter.counter().current());

        let mut i = 0;
        while let Some(next) = iter.fetch_one() {
            let value = next.value + 0usize;
            assert_eq!(value, i);
            i += 1;
            assert_eq!(i, iter.counter().current());
        }
    }

    pub(crate) fn atomic_fetch_n<A>(iter: A, n: usize)
    where
        A: AtomicIter,
        A::Item: Add<usize, Output = usize>,
    {
        assert_eq!(0, iter.counter().current());

        let mut i = 0;
        let mut has_more = true;

        while has_more {
            has_more = false;
            let next_id_and_chunk = iter.fetch_n(n);
            let begin_idx = next_id_and_chunk.begin_idx();
            for (j, value) in next_id_and_chunk.values().enumerate() {
                let value = value + 0usize;
                assert_eq!(value, begin_idx + j);
                assert_eq!(value, i);

                i += 1;

                has_more = true;
            }
        }
    }

    pub(crate) fn atomic_exact_fetch_one<A>(iter: A)
    where
        A: AtomicIterWithInitialLen,
        A::Item: Add<usize, Output = usize>,
    {
        let mut remaining = ATOMIC_TEST_LEN;

        assert!(!iter.is_empty());
        assert_eq!(iter.len(), remaining);

        while iter.fetch_one().is_some() {
            remaining -= 1;
            assert_eq!(iter.len(), remaining);
        }

        assert_eq!(iter.len(), 0);
        assert!(iter.is_empty());
    }

    pub(crate) fn atomic_exact_fetch_n<A>(iter: A, n: usize)
    where
        A: AtomicIterWithInitialLen,
        A::Item: Add<usize, Output = usize>,
    {
        let mut remaining = ATOMIC_TEST_LEN;

        assert!(!iter.is_empty());
        assert_eq!(iter.len(), remaining);

        let mut has_more = true;
        while has_more {
            has_more = false;

            let next_id_and_chunk = iter.fetch_n(n);
            if next_id_and_chunk.values().next().is_some() {
                has_more = true;
            }

            if n > remaining {
                remaining = 0;
            } else {
                remaining -= n;
            }

            assert_eq!(iter.len(), remaining);
        }

        assert_eq!(iter.len(), 0);
        assert!(iter.is_empty());
    }
}
