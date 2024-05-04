use super::{
    buffered::{buffered_chunk::BufferedChunk, buffered_iter::BufferedIter},
    default_fns,
    wrappers::values::ConIterValues,
};
use crate::{
    next::{Next, NextChunk},
    ConIterIdsAndValues,
};

/// Trait defining a concurrent iterator with `next` and `next_id_and_chunk` methods which can safely be called my multiple threads concurrently.
pub trait ConcurrentIter: Send + Sync {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Type of the buffered iterator returned by the `chunk_iter` method when elements are fetched in chunks by each thread.
    type BufferedIter: BufferedChunk<Self::Item, ConIter = Self>;

    /// Advances the iterator and returns the next value together with its enumeration index.
    ///
    /// Returns [None] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let num_threads = 4;
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    ///
    /// let outputs = ConcurrentBag::new();
    ///
    /// let con_iter = &slice.con_iter();
    /// let bag = &outputs;
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             while let Some(next) = con_iter.next_id_and_value() {
    ///                 let expected_value = char::from_digit(next.idx as u32, 10).unwrap();
    ///                 assert_eq!(next.value, &expected_value);
    ///
    ///                 bag.push(*next.value);
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut outputs: Vec<char> = outputs.into_inner().into();
    /// outputs.sort();
    /// assert_eq!(characters, outputs);
    /// ```
    fn next_id_and_value(&self) -> Option<Next<Self::Item>>;

    /// Advances the iterator `chunk_size` times and returns an iterator of at most `chunk_size` consecutive next values.
    /// Further, the beginning enumeration index of the yielded values is returned.
    ///
    /// This method:
    /// * returns an iterator of `chunk_size` elements if there exist sufficient elements left in the iteration, or
    /// * it might return an iterator of `m < chunk_size` elements if there exists only `m` elements left, or
    /// * it might return None, it will never return Some of an empty iterator.
    ///
    /// This call would be equivalent to calling `next_id_and_value` method `chunk_size` times in a single-threaded execution.
    /// However, calling `next` method `chunk_size` times in a concurrent execution does not guarantee to return `chunk_size` consecutive elements.
    /// On the other hand, `next_chunk` guarantees that it returns consecutive elements, preventing any intermediate calls.
    ///
    /// More importantly, this is an important performance optimization feature that enables
    /// * reducing the number of atomic updates,
    /// * avoiding performance degradation by false sharing if the fetched inputs will be processed and written to a shared memory (see [`ConcurrentBag` performance notes](https://docs.rs/orx-concurrent-bag/latest/orx_concurrent_bag/#section-performance-notes)).
    ///
    /// # Examples
    ///
    /// Note that `next_chunk` method returns a [`NextChunk`].
    /// Further, [`NextChunk::values`] is a regular `ExactSizeIterator` which might yield at most `chunk_size` elements.
    /// Note that the iterator will never be empty.
    /// Whenever the concurrent iterator is used up, `next_chunk` method, similar to `next`, will return `None`.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let num_threads = 4;
    ///
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    /// let outputs = ConcurrentBag::new();
    ///
    /// let iter = &slice.con_iter();
    /// let bag = &outputs;
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             while let Some(chunk) = iter.next_chunk(2) {
    ///                 for (i, value) in chunk.values.enumerate() {
    ///                     // we have access to the original index in `slice`
    ///                     let idx = chunk.begin_idx + i;
    ///                     let expected_value = char::from_digit(idx as u32, 10).unwrap();
    ///                     assert_eq!(value, &expected_value);
    ///
    ///                     bag.push(*value);
    ///                 }
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut outputs: Vec<char> = outputs.into_inner().into();
    /// outputs.sort();
    /// assert_eq!(characters, outputs);
    /// ```
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>>;

    /// Creates an iterator which pulls elements from this iterator as chunks of the given `chunk_size`.
    ///
    /// Returned iterator is a regular `Iterator`, except that it is linked to the concurrent iterator and pulls its elements concurrently from the parent iterator.
    /// The `next` call of the buffered iterator returns `None` if the concurrent iterator is consumed.
    /// Otherwise, it returns a [`NextChunk`] which is composed of two values:
    /// * `begin_idx`: the index in the original source data, or concurrent iterator, of the first element of the pulled chunk,
    /// * `values`: an `ExactSizeIterator` containing at least 1 and at most `chunk_size` consecutive elements pulled from the original source data.
    ///
    /// Iteration in chunks is allocation free whenever possible; for instance, when the source data is a vector, a slice or an array, etc. with known size.
    /// On the other hand, when the source data is an arbitrary `Iterator`, iteration in chunks requires a buffer to write the chunk of pulled elements.
    ///
    /// This method is memory efficient in these situations.
    /// It allocates a buffer of `chunk_size` only once when created, only if the source data requires it.
    /// This buffer is used over and over until the concurrent iterator is consumed.
    ///
    /// # Example
    ///
    /// Example below illustrates a parallel sum operation.
    /// Entire iteration requires an allocation (4 threads) * (16 chunk size) = 64 elements.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 4;
    ///
    /// let numbers = (0..16384).map(|x| x * 2);
    /// let iter = &numbers.into_con_iter();
    ///
    /// let sum = std::thread::scope(|s| {
    ///     let mut handles = vec![];
    ///     for _ in 0..num_threads {
    ///         handles.push(s.spawn(move || {
    ///             let mut buffered = iter.buffered_iter(16);
    ///             let mut sum = 0;
    ///             while let Some(chunk) = buffered.next() {
    ///                 sum += chunk.values.sum::<usize>();
    ///             }
    ///             sum
    ///         }));
    ///     }
    ///
    ///     handles
    ///         .into_iter()
    ///         .map(|x| x.join().expect("-"))
    ///         .sum::<usize>()
    /// });
    ///
    /// let expected = 16384 * 16383;
    /// assert_eq!(sum, expected);
    /// ```
    ///
    /// # `buffered_iter` and `next_chunk`
    ///
    /// When iterating over single elements using `next` or `values`, the concurrent iterator just yields the element without requiring any allocation.
    ///
    /// When iterating as chunks, concurrent iteration might or might not require an allocation.
    /// * For instance, no allocation is required if the source data of the iterator is a vector, a slice or an array, etc.
    /// * On the other hand, an allocation of `chunk_size` is required if the source data is an any `Iterator` without further type information.
    ///
    /// Pulling elements as chunks using the `next_chunk` or `buffered_iter` methods differ in the latter case as follows:
    /// * `next_chunk` will allocate a vector of `next_chunk` elements every time it is called;
    /// * `buffered_iter` will allocate a vector of `next_chunk` only once on construction, and this buffer will be used over and over until the concurrent iterator is consumed leading to negligible allocation.
    ///
    /// Therefore, it is safer to always use `buffered_iter`, unless we need to keep changing the `chunk_size` through iteration, which is a rare scenario.
    ///
    /// In a typical use case where we concurrently iterate over the elements of the iterator using `num_threads` threads:
    /// * we will create `num_threads` buffered iterators; i.e., we will call `buffered_iter` once from each thread,
    /// * each thread will allocate a vector of `chunk_size` capacity.
    ///
    /// In total, the iteration will use an additional memory of `num_threads * chunk_size`.
    /// Note that the amount of allocation is not a function of the length of the source data.
    /// Assuming the length will be large in a scenario requiring parallel iteration, the allocation can be considered to be very small.
    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter>;

    /// Advances the iterator and returns the next value.
    ///
    /// Returns [None] when iteration is finished.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let num_threads = 4;
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    ///
    /// let outputs = ConcurrentBag::new();
    ///
    /// let con_iter = &slice.con_iter();
    /// let bag = &outputs;
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             while let Some(value) = con_iter.next() {
    ///                 bag.push(*value);
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut outputs: Vec<char> = outputs.into_inner().into();
    /// outputs.sort();
    /// assert_eq!(characters, outputs);
    /// ```
    #[inline(always)]
    fn next(&self) -> Option<Self::Item> {
        self.next_id_and_value().map(|x| x.value)
    }

    /// Returns an `Iterator` over the values of elements of the concurrent iterator.
    ///
    /// Note that `values` method can be called concurrently from multiple threads to create multiple local-to-thread regular `Iterator`s.
    /// However, each of these iterators will be connected in the sense that:
    /// * all iterators will be aware of the progress by the other iterators;
    /// * each element will be yielded exactly once.
    ///
    /// The iterator's `next` method does nothing but call the `next`; this iterator is only to allow for using `for` loops directly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let num_threads = 4;
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    ///
    /// let outputs = ConcurrentBag::new();
    ///
    /// let con_iter = &slice.con_iter();
    /// let bag = &outputs;
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             for value in con_iter.values() {
    ///                 bag.push(*value);
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// // parent concurrent iterator is completely consumed
    /// assert!(con_iter.values().next().is_none());
    ///
    /// let mut outputs: Vec<char> = outputs.into_inner().into();
    /// outputs.sort();
    /// assert_eq!(characters, outputs);
    /// ```
    fn values(&self) -> ConIterValues<Self>
    where
        Self: Sized,
    {
        self.into()
    }

    /// Returns an `Iterator` over the ids and values of elements of the concurrent iterator.
    ///
    /// Note that `values` method can be called concurrently from multiple threads to create multiple local-to-thread regular `Iterator`s.
    /// However, each of these iterators will be connected in the sense that:
    /// * all iterators will be aware of the progress by the other iterators;
    /// * each element will be yielded exactly once.
    ///
    /// The iterator's `next` method does nothing but call the `next_id_and_value`; this iterator is only to allow for using `for` loops directly.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let num_threads = 4;
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    ///
    /// let outputs = ConcurrentBag::new();
    ///
    /// let con_iter = &slice.con_iter();
    /// let bag = &outputs;
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             for (idx, value) in con_iter.ids_and_values() {
    ///                 let expected_value = char::from_digit(idx as u32, 10).unwrap();
    ///                 assert_eq!(value, &expected_value);
    ///
    ///                 bag.push(*value);
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// // parent concurrent iterator is completely consumed
    /// assert!(con_iter.ids_and_values().next().is_none());
    ///
    /// let mut outputs: Vec<char> = outputs.into_inner().into();
    /// outputs.sort();
    /// assert_eq!(characters, outputs);
    /// ```
    fn ids_and_values(&self) -> ConIterIdsAndValues<Self>
    where
        Self: Sized,
    {
        self.into()
    }

    /// Applies the function `fun` to each element of the iterator concurrently.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// At each iteration of the loop, this method pulls `chunk_size` elements from the iterator.
    /// Under the hood, this method will loop using the buffered chunks iterator, pulling items as batches of the given `chunk_size`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let chunk_size = 2;
    /// let num_threads = 4;
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    ///
    /// let outputs = ConcurrentBag::new();
    ///
    /// let con_iter = &slice.con_iter();
    /// let bag = &outputs;
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             con_iter.for_each(chunk_size, |c| {
    ///                 bag.push(c.to_digit(10).unwrap());
    ///             });
    ///         });
    ///     }
    /// });
    ///
    /// // parent concurrent iterator is completely consumed
    /// assert!(con_iter.next().is_none());
    ///
    /// let sum: u32 = outputs.into_inner().iter().sum();
    /// assert_eq!(sum, (0..8).sum());
    /// ```
    fn for_each<Fun: FnMut(Self::Item)>(&self, chunk_size: usize, fun: Fun)
    where
        Self: Sized,
    {
        default_fns::for_each::for_each(self, chunk_size, fun);
    }

    /// Applies the function `fun` to each index and corresponding element of the iterator concurrently.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// At each iteration of the loop, this method pulls `chunk_size` elements from the iterator.
    /// Under the hood, this method will loop using the buffered chunks iterator, pulling items as batches of the given `chunk_size`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let chunk_size = 2;
    /// let num_threads = 4;
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    /// let slice = characters.as_slice();
    ///
    /// let outputs = ConcurrentBag::new();
    ///
    /// let con_iter = &slice.con_iter();
    /// let bag = &outputs;
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             con_iter.enumerate_for_each_n(chunk_size, |i, c| {
    ///                 let expected_value = char::from_digit(i as u32, 10).unwrap();
    ///                 assert_eq!(c, &expected_value);
    ///
    ///                 bag.push(c.to_digit(10).unwrap());
    ///             });
    ///         });
    ///     }
    /// });
    ///
    /// // parent concurrent iterator is completely consumed
    /// assert!(con_iter.next().is_none());
    ///
    /// let sum: u32 = outputs.into_inner().iter().sum();
    /// assert_eq!(sum, (0..8).sum());
    /// ```
    fn enumerate_for_each_n<Fun: FnMut(usize, Self::Item)>(&self, chunk_size: usize, fun: Fun)
    where
        Self: Sized,
    {
        default_fns::for_each::for_each_with_ids(self, chunk_size, fun)
    }

    /// Folds the elements of the iterator pulled concurrently using `fold` function.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// Each thread will start its concurrent fold operation with the `neutral` value.
    /// This value is then transformed into the result by applying the `fold` on it together with the pulled elements.
    ///
    /// Therefore, each thread will end up at a different partial result.
    /// Further, each thread's partial result might be different in every execution.
    ///
    /// However, once `fold` is applied starting again from `neutral` using the thread results, we compute the deterministic result.
    /// This establishes a very ergonomic parallel fold implementation.
    ///
    /// # Chunk Size
    ///
    /// When `chunk_size` is set to 1, threads will pull elements from the iterator one by one.
    /// This might be the preferred setting when we are working with general `ConcurrentIter`s rather than `ExactSizeConcurrentIter`s,
    /// or whenever the work to be done to `fold` the results is large enough to make the time spent for updating atomic counters insignificant.
    ///
    /// On the other hand, whenever we are working with an `ExactSizeConcurrentIter` and the iterator has many elements,
    /// it is possible to avoid the cost of atomic updates almost completely by setting the chunk size to a larger value.
    /// Rule of thumb in this situation is that the larger the better provided that the chunk size is not too large that would lead some threads to remain idle.
    /// However, note that this optimization is only significant if the `fold` operation is significantly small, such as the addition example below.
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is zero; chunk size is required to be strictly positive.
    ///
    /// # Examples
    ///
    /// Notice that the initial value is called `neutral` as in **monoids**, rather than init or initial.
    /// This is to highlight that each thread will start its separate execution from this value.
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
    /// * `""` for string concat
    /// * `[]` for list concat
    /// * `true` & logical, etc.
    ///
    /// ***Wrong Example with a Non-Neutral Element***
    ///
    /// In a sequential fold operation, one can choose to start the summation above with an initial value of 100.
    /// Then, the resulting value would deterministically be 117.
    ///
    /// However, if we pass 100 as the neutral element to the concurrent fold above, we would receive 217 (additional 100 for each thread).
    /// Notice that the result depends on the number of threads used in computation.
    /// This is incorrect.
    ///
    /// In either case, it is a good practice to leave 100 out of the fold operation.
    /// It is preferable to pass 0 as the initial and neutral element, and add 100 to the result of the fold operation.
    fn fold<B, Fold>(&self, chunk_size: usize, neutral: B, fold: Fold) -> B
    where
        Fold: FnMut(B, Self::Item) -> B,
        Self: Sized,
    {
        default_fns::fold::fold(self, chunk_size, fold, neutral)
    }

    /// Returns Some of the remaining length of the iterator if it is known; returns None otherwise.
    fn try_get_len(&self) -> Option<usize>;
}

/// A concurrent iterator that knows its exact length.
pub trait ExactSizeConcurrentIter: ConcurrentIter {
    /// Returns the exact remaining length of the concurrent iterator.
    fn len(&self) -> usize;

    /// Returns true if the iterator is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
