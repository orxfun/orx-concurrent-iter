use super::atomic_counter::AtomicCounter;
use crate::{next::Next, NextChunk, NextManyExact};

/// Trait defining a set of concurrent iterators which internally uses an atomic counter of elements to be yielded.
///
/// Note that every `A: AtomicIter` also implements `ConcurrentIter`.
pub trait AtomicIter<T: Send + Sync>: Send + Sync {
    /// Returns a reference to the underlying advanced item counter.
    fn counter(&self) -> &AtomicCounter;

    /// Returns the `item_idx`-th item that the iterator yields; returns None if the iterator completes before.
    fn get(&self, item_idx: usize) -> Option<T>;

    /// Returns the next item that the iterator yields; returns None if the iteration has completed.
    #[inline(always)]
    fn fetch_one(&self) -> Option<Next<T>> {
        let idx = self.counter().fetch_and_increment();
        self.get(idx).map(|value| Next { idx, value })
    }

    /// Returns an iterator of the next `n` **consecutive items** that the iterator yields.
    /// It might return an iterator of less or no items if the iteration does not have sufficient elements left.
    fn fetch_n(&self, n: usize) -> impl NextChunk<T>;

    /// Applies the function `f` to each element of the iterator concurrently.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// Calling thread will process one input at a time ([`AtomicIter::fetch_one`]) when `n` is set to 1.
    /// Alternatively, each thread might process `n` consecutive elements at each iteration ([`AtomicIter::fetch_n`]).
    fn for_each_n<F: FnMut(T)>(&self, n: usize, f: F);

    /// Applies the function `f` to each element and its corresponding index of the iterator concurrently.
    /// It may be considered as the concurrent counterpart of the chain of `enumerate` and `for_each_n` calls.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// Calling thread will process one input at a time ([`AtomicIter::fetch_one`]) when `n` is set to 1.
    /// Alternatively, each thread might process `n` consecutive elements at each iteration ([`AtomicIter::fetch_n`]).
    fn enumerate_for_each_n<F: FnMut(usize, T)>(&self, n: usize, f: F);
}

/// An atomic counter based iterator with exactly known initial length.
pub trait AtomicIterWithInitialLen<T: Send + Sync>: AtomicIter<T> {
    /// Returns the initial length of the atomic iterator.
    fn initial_len(&self) -> usize;

    /// Returns an iterator of the next `n` **consecutive items** that the iterator together with an exact size iterator.
    /// It might return an iterator of less or no items if the iteration does not have sufficient elements left.
    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<T, impl ExactSizeIterator<Item = T>>;
}
