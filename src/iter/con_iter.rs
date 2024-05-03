use super::wrappers::values::ConIterValues;
use crate::{next::Next, ConIterIdsAndValues, NextChunk, NextManyExact};

/// Trait defining a concurrent iterator with `next` and `next_id_and_chunk` methods which can safely be called my multiple threads concurrently.
pub trait ConcurrentIter: Send + Sync {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

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
    /// * returns an iterator of `chunk_size` elements if there exists sufficient elements left in the iteration, or
    /// * it might return an iterator of `m < chunk_size` elements if there exists only `m` elements left, or
    /// * it might return an empty iterator.
    ///
    /// This call would be equivalent to calling `next_id_and_value` method `chunk_size` times in a single-threaded execution.
    /// However, calling `next` method `chunk_size` times in a concurrent execution does not guarantee to return `chunk_size` consecutive elements.
    /// On the other hand, `next_id_and_chunk` guarantees that it returns consecutive elements, preventing any intermediate calls.
    ///
    /// # Examples
    ///
    /// Note that `next_chunk` method returns a [`NextChunk`].
    /// Further, [`NextChunk::values`] is a regular `Iterator` which might yield at most `chunk_size` elements.
    /// However, when the iterator is consumed concurrently, it will return an iterator which yields no elements.
    /// Due to this, looping using the `next_chunk` method does not allow using `while let Some` pattern, and hence, it is not as ergonomic.
    /// Therefore, whenever the length of the iterator is known; i.e., whenever the iterator also implements [`ExactSizeConcurrentIter`], [`ExactSizeConcurrentIter::next_exact_chunk`] is preferable.
    /// Alternatively, [`ConcurrentIter::for_each_n`] or [`ConcurrentIter::enumerate_for_each_n`] methods can be used to avoid handling the loop manually.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let num_threads = 4;
    ///
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
    ///             loop {
    ///                 let mut has_any_more = false;
    ///
    ///                 let next = con_iter.next_chunk(2);
    ///                 let begin = next.begin_idx();
    ///                 for (i, value) in next.values().enumerate() {
    ///                     has_any_more = true;
    ///                     let idx = begin + i;
    ///                     let expected_value = char::from_digit(idx as u32, 10).unwrap();
    ///                     assert_eq!(value, &expected_value);
    ///
    ///                     bag.push(*value);
    ///                 }
    ///
    ///                 if !has_any_more {
    ///                     break;
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
    fn next_chunk(&self, chunk_size: usize) -> impl NextChunk<Self::Item>;

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
    /// Under the hood, this method will loop using the [`ConcurrentIter::next_chunk`] method, pulling items `chunk_size` by `chunk_size`.
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
    ///             con_iter.for_each_n(chunk_size, |c| {
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
    fn for_each_n<Fun: FnMut(Self::Item)>(&self, chunk_size: usize, fun: Fun);

    /// Applies the function `fun` to each index and corresponding element of the iterator concurrently.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// At each iteration of the loop, this method pulls `chunk_size` elements from the iterator.
    /// Under the hood, this method will loop using the [`ConcurrentIter::next_chunk`] method, pulling items `chunk_size` by `chunk_size`.
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
    fn enumerate_for_each_n<Fun: FnMut(usize, Self::Item)>(&self, chunk_size: usize, fun: Fun);

    /// Applies the function `fun` to each element of the iterator concurrently.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// Under the hood, this method will loop using the [`ConcurrentIter::next`] method, pulling items one by one.
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
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             con_iter.for_each(|c| {
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
    fn for_each<Fun: FnMut(Self::Item)>(&self, fun: Fun) {
        self.for_each_n(1, fun)
    }

    /// Applies the function `fun` to each index and corresponding element of the iterator concurrently.
    /// It may be considered as the concurrent counterpart of the chain of `enumerate` and `for_each` calls.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// Under the hood, this method will loop using the [`ConcurrentIter::next_id_and_value`] method, pulling items one by one.
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
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(move || {
    ///             con_iter.enumerate_for_each(|i, c| {
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
    fn enumerate_for_each<Fun: FnMut(usize, Self::Item)>(&self, fun: Fun) {
        self.enumerate_for_each_n(1, fun)
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
    /// * empty string/list & string/list concat
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
        Fold: FnMut(B, Self::Item) -> B;

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

    /// Returns the next chunk with the requested `chunk_size`:
    /// * Returns `None` if there are no more elements to yield.
    /// * Returns `Some` of a [`crate::NextManyExact`] which contains the following information:
    ///   * `begin_idx`: the index of the first element to be yielded by the `values` iterator.
    ///   * `values`: an `ExactSizeIterator` with known `len` which is guaranteed to be positive and less than or equal to `chunk_size`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let chunk_size = 2;
    /// let num_threads = 4;
    ///
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
    ///             while let Some(next) = con_iter.next_exact_chunk(chunk_size) {
    ///                 let begin = next.begin_idx();
    ///                 for (i, value) in next.values().enumerate() {
    ///                     let idx = begin + i;
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
    fn next_exact_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>>>;
}
