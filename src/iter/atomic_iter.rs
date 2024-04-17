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
