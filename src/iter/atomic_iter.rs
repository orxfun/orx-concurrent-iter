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

    /// Folds the elements of the iterator pulled concurrently using `fold` function.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// Each thread will start its concurrent fold operation with the `neutral` value.
    /// This value is then transformed into the result by applying the `fold` on it together with the pulled elements.
    ///
    /// Therefore, each thread will end up at a different partial result.
    /// Further, each thread's partial result might be different in every execution.
    /// However, once `fold` is applied starting again from `neutral` using the thread results will lead to the deterministic result which would have been obtained in sequential operation.
    /// This establishes a very ergonomic parallel fold implementation.
    ///
    /// # Examples
    ///
    /// Notice that the initial value is called `neutral` as in **monoids**, rather than init or initial.
    /// This is to highlight that each thread will start its separate execution from this value.
    ///
    /// ### Good Example with a Neutral Element
    ///
    /// Integer addition and number zero are good examples for `neutral` and `fold`, respectively.
    /// Assume our iterator will yield 4 values: [3, 4, 1, 9].
    /// We want to sum these values using two threads.
    /// We can achieve parallelism very conveniently using `fold` as follows.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 2;
    ///
    /// let numbers = vec![3, 4, 1, 9];
    /// let slice = numbers.as_slice();
    /// let iter = &slice.con_iter();
    ///
    /// let neutral = 0; // neutral for i32 & add
    ///
    /// let sum = std::thread::scope(|s| {
    ///     (0..num_threads)
    ///         .map(|_| s.spawn(move || iter.fold(1, neutral, |x, y| x + y))) // parallel fold
    ///         .map(|x| x.join().unwrap())
    ///         .fold(neutral, |x, y| x + y) // sequential fold
    /// });
    ///
    /// assert_eq!(sum, 17);
    /// ```
    ///
    /// Note that this code can execute in one of many possible ways.
    /// Let's say our two threads are called tA and tB.
    /// * tA might pull and sum all four of the numbers; hence, returns 17. tB cannot pull any element and just returns the neutral element. Sequential fold will add 17 and 0, and return 17.
    /// * tA might pull only the third element; hence, returns 0+1 = 1. tB pulls the other 3 elements and returns 0+3+4+9 = 16. Final fold will then return 0+1+16 = 17.
    /// * and so on, so forth.
    ///
    /// `ConcurrentIter` guarantees that each element is visited and computed exactly once.
    /// Therefore, the parallel computation will always be correct provided that we provide a neutral element such that:
    ///
    /// ```rust ignore
    /// assert_eq!(fold(neutral, element), element);
    /// ```
    ///
    /// Other trivial examples are:
    /// * `1` & multiplication
    /// * empty string/list & string/list concat
    /// * `true` & logical, etc.
    ///
    /// ## Wrong Example with a Non-Neutral Element
    ///
    /// In a sequential fold operation, once can start the summation above with an initial value of 100.
    /// Then, the resulting value would deterministically be 117.
    ///
    /// However, if we pass 100 as the neutral element to the concurrent fold above, we would receive 217 (additional 100 for each thread).
    /// Notice that the result depends on the number of threads used in computation.
    /// This is incorrect.
    ///
    /// In either case, it is a good practice to leave 100 out of the fold operation.
    /// Ideally, we would pass 0 as the initial and neutral element, and add 100 to the result of the fold operation.
    fn fold<B, Fold: FnMut(B, T) -> B>(&self, chunk_size: usize, neutral: B, fold: Fold) -> B;
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
