use super::atomic_counter::AtomicCounter;
use crate::next::{Next, NextChunk};

/// Trait defining a set of concurrent iterators which internally uses an atomic counter of elements to be yielded.
///
/// Note that every `A: AtomicIter` also implements `ConcurrentIter`.
pub trait AtomicIter<T: Send + Sync>: Send + Sync {
    /// Returns a reference to the underlying advanced item counter.
    fn counter(&self) -> &AtomicCounter;

    /// Progresses the atomic counter by `number_to_fetch` elements and returns the beginning index of the elements to be fetched.
    /// Returns None if the iterator is terminated and there are no more elements to be fetched.
    ///
    /// Note that this method must only return when the calling thread is allowed to fetch elements.
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize>;

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
    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>>;

    /// Skips all remaining elements of the iterator and assumes that the end of the iterator is reached.
    ///
    /// This method establishes a very primitive, convenient and critical communication among threads for search scenarios with an early exit condition.
    /// Assume, for instance, that we are trying to `find` an element satisfying a predicate using multiple threads.
    /// Whenever a threads finds a match, it can call this method and return the found value.
    /// Then, when the other threads try to pull next element from the iterator, they will observe that the iterator has ended.
    /// Therefore, they will as well return early as desired.
    fn early_exit(&self);
}

/// An atomic counter based iterator with exactly known initial length.
pub trait AtomicIterWithInitialLen<T: Send + Sync>: AtomicIter<T> {
    /// Returns the initial length of the atomic iterator.
    fn initial_len(&self) -> usize;
}
