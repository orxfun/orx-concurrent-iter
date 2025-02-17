use super::{
    buffered::{buffered_chunk::BufferedChunkX, buffered_iter::BufferedIter},
    default_fns,
    wrappers::values_x::ConIterValuesX,
};
use crate::has_more::HasMore;

/// Trait defining a concurrent iterator with `next` and `next_id_and_chunk` methods which can safely be called my multiple threads concurrently.
///
/// Note that type or method names with suffix 'x' is an indicator of the dropped promise of keeping order.
/// * `ConcurrentIterX` allows concurrent iteration; however, it may or may not provide the initial order of
///   an element in the providing source.
/// * `ConcurrentIter` extends `ConcurrentIterX` by including the keeping track of order guarantee.
///
/// This crate provides ordered `ConcurrentIter` implementations of all iterators.
/// In other words, `ConcurrentIterX` implementations already preserve the input order and hence
/// the iterators also implement `ConcurrentIter`.
/// A different `ConcurrentIterX` implementation is provided only if it makes it possible to have a performance gain.
/// One example is concurrent iterators created for arbitrary sequential iterators, in which case keeping track of
/// input order might have an impact.
///
/// In such cases, in order to make the choice explicit, these traits are differentiated.
///
/// In most of the practical use cases, we require only the `ConcurrentIter`.
/// The differentiation is utilized by under the hood coordination of crates aiming high performance
/// such as [orx_parallel](https://crates.io/crates/orx-parallel).
///
/// # Examples
///
/// **A. When order matters**
///
/// Once can create a concurrent iterator from a sequential iterator and use it as it is.
/// This will guarantee to provide the correct index of the input.
/// For instance in the following, `(i, value)` pair provided by `ids_and_values`
/// will guarantee that `2*i == value`.
///
/// ```rust
/// use orx_concurrent_iter::*;
/// use orx_concurrent_bag::ConcurrentBag;
///
/// let num_threads = 8;
///
/// let seq_iter = (0..1024).map(|x| x * 2).into_iter();
/// let con_iter = seq_iter.into_con_iter();
///
/// let con_bag = ConcurrentBag::new();
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             for (i, value) in con_iter.ids_and_values() {
///                 con_bag.push((i, value / 2));
///             }
///         });
///     }
/// });
///
/// let vec = con_bag.into_inner();
/// for (i, value) in vec.into_iter() {
///     assert_eq!(i, value as usize);
/// }
/// ```
///
/// **B. When order does not matter**
///
/// Alternatively, if the order does not matter, we can create an unordered version of the concurrent iterator.
/// We can directly create one using `into_con_iter_x` on the source type.
/// Alternatively, we can convert any ordered concurrent iterator any time by calling `into_con_iter_x`.
/// In the following, we use an unordered version, since we are not interested in the indices of elements
/// while computing a parallel sum.
///
/// ```rust
/// use orx_concurrent_iter::*;
///
/// let num_threads = 8;
///
/// let seq_iter = (0..1024).map(|x| x * 2).into_iter();
/// let con_iter = seq_iter.into_con_iter_x(); // directly into x
/// // let con_iter = seq_iter.into_con_iter().into_con_iter_x(); // or alternatively
///
/// let sum: i32 = std::thread::scope(|s| {
///     let mut handles: Vec<_> = vec![];
///     for _ in 0..num_threads {
///         // as expected, `ids_and_values` method is not available
///         // but we have `values`
///         handles.push(s.spawn(|| con_iter.values().sum::<i32>()));
///     }
///
///     handles.into_iter().map(|x| x.join().expect("-")).sum()
/// });
///
/// assert_eq!(sum, (0..1024).sum::<i32>() * 2)
/// ```
pub trait ConcurrentIterX: Send + Sync {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    /// Type of the buffered iterator returned by the `buffered_iter_x` method when elements are fetched in chunks by each thread.
    type BufferedIterX: BufferedChunkX<Self::Item, ConIter = Self>;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    fn into_seq_iter(self) -> Self::SeqIter;

    /// Advances the iterator `chunk_size` times and returns an iterator of at most `chunk_size` consecutive next values.
    ///
    /// See [`next_chunk`] for a version that additionally provides the input order of the elements in the chunk.
    ///
    /// [`next_chunk`]: crate::ConcurrentIter::next_chunk
    ///
    /// This method:
    /// * returns an iterator of `chunk_size` elements if there exist sufficient elements left in the iteration, or
    /// * it might return an iterator of `m < chunk_size` elements if there exists only `m` elements left, or
    /// * it might return None
    /// * => it will never return Some of an empty iterator.
    ///
    /// This call would be equivalent to calling `next_id_and_value` method `chunk_size` times in a single-threaded execution.
    /// However, calling `next` method `chunk_size` times in a concurrent execution does not guarantee to return `chunk_size` consecutive elements.
    /// On the other hand, `next_chunk_x` guarantees that it returns consecutive elements, preventing any intermediate calls.
    ///
    /// More importantly, this is an important performance optimization feature that enables
    /// * reducing the number of atomic updates,
    /// * avoiding performance degradation by false sharing if the fetched inputs will be processed and written to a shared memory (see [`ConcurrentBag` performance notes](https://docs.rs/orx-concurrent-bag/latest/orx_concurrent_bag/#section-performance-notes)).
    ///
    /// # Examples
    ///
    /// Note that `next_chunk_x` method returns a regular `ExactSizeIterator` which might yield at most `chunk_size` elements.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    ///
    /// let (num_threads, chunk_size) = (4, 32);
    /// let strings: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    /// let bag = ConcurrentBag::new();
    ///
    /// let iter = strings.con_iter();
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             while let Some(chunk) = iter.next_chunk_x(chunk_size) {
    ///                 // iterate in chunks; each chunk is an ExactSizeIterator
    ///                 assert!((0..=chunk_size).contains(&chunk.len()));
    ///
    ///                 let mapped = chunk.map(|x| format!("{}!", x));
    ///                 bag.extend(mapped);
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut collected = bag.into_inner();
    /// collected.sort();
    ///
    /// let mut expected: Vec<_> = (0..1024).map(|x| format!("{}!", x)).collect();
    /// expected.sort();
    ///
    /// assert_eq!(expected, collected);
    /// ```
    fn next_chunk_x(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = Self::Item>>;

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
    /// fn to_str(num: usize) -> String {
    ///     num.to_string()
    /// }
    ///
    /// let num_threads = 4;
    /// let strings: Vec<_> = (0..1024).map(to_str).collect();
    /// let bag = ConcurrentBag::new();
    ///
    /// let iter = strings.con_iter();
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             while let Some(value) = iter.next() {
    ///                 bag.push(value.len());
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let outputs: Vec<_> = bag.into_inner().into();
    /// assert_eq!(outputs.len(), strings.len());
    /// ```
    fn next(&self) -> Option<Self::Item>;

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
    /// fn to_str(num: usize) -> String {
    ///     num.to_string()
    /// }
    ///
    /// let num_threads = 4;
    /// let strings: Vec<_> = (0..1024).map(to_str).collect();
    /// let bag = ConcurrentBag::new();
    ///
    /// let iter = strings.con_iter();
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             for value in iter.values() {
    ///                 bag.push(value.len());
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let outputs: Vec<_> = bag.into_inner().into();
    /// assert_eq!(outputs.len(), strings.len());
    /// ```
    fn values(&self) -> ConIterValuesX<Self>
    where
        Self: Sized,
    {
        self.into()
    }

    /// Skips all remaining elements of the iterator and assumes that the end of the iterator is reached.
    ///
    /// This method establishes a very primitive, convenient and critical communication among threads for search scenarios with an early exit condition.
    /// Assume, for instance, that we are trying to `find` an element satisfying a predicate using multiple threads.
    /// Whenever a threads finds a match, it can call this method and return the found value.
    /// Then, when the other threads try to pull next element from the iterator, they will observe that the iterator has ended.
    /// Therefore, they will as well return early as desired.
    ///
    /// # Examples
    ///
    /// As discussed above, a straightforward use case is the `find` iterator method. You may find a very simple & convenient example implementation below.
    /// Likewise, it might be used to stop an infinite parallel search; for instance, when trying to find a feasible solution to an optimization problem using a randomized search.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// fn par_find<I, P>(iter: I, predicate: P, n_threads: usize) -> Option<(usize, I::Item)>
    /// where
    ///     I: ConcurrentIter,
    ///     P: Fn(&I::Item) -> bool + Send + Sync,
    /// {
    ///     std::thread::scope(|s| {
    ///         (0..n_threads)
    ///             .map(|_| {
    ///                 s.spawn(|| {
    ///                     for (i, x) in iter.ids_and_values() {
    ///                         if predicate(&x) {
    ///                             iter.skip_to_end();
    ///                             return Some((i, x));
    ///                         }
    ///                     }
    ///                     None
    ///                 })
    ///             })
    ///             .collect::<Vec<_>>()
    ///             .into_iter()
    ///             .flat_map(|x| x.join().expect("failed to join thread"))
    ///             .min_by_key(|x| x.0)
    ///     })
    /// }
    ///
    /// let mut names: Vec<_> = (0..8785).map(|x| x.to_string()).collect();
    /// names[42] = "foo".to_string();
    ///
    /// let result = par_find(names.con_iter(), |x| x.starts_with('x'), 4);
    /// assert_eq!(result, None);
    ///
    /// let result = par_find(names.con_iter(), |x| x.starts_with('f'), 4);
    /// assert_eq!(result, Some((42, &"foo".to_string())));
    /// ```
    ///
    /// Notice that in the example above, only one among 8785 elements satisfies the predicate.
    /// If the thread that finds "foo" did not call `skip_to_end`, the other threads would traverse through all elements and check the condition.
    /// On the other hand, once `skip_to_end` is called, none of the threads can find any more elements to pull, and hence, immediately exit in their next `next` call.
    fn skip_to_end(&self);

    /// Applies the function `fun` to each element of the iterator concurrently.
    ///
    /// Note that this method might be called on the same iterator at the same time from different threads.
    /// The iterator guarantees that the function is applied to each element exactly once.
    ///
    /// At each iteration of the loop, this method pulls `chunk_size` elements from the iterator.
    /// Under the hood, this method will loop using the buffered chunks iterator, pulling items as batches of the given `chunk_size`.
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is zero; chunk size is required to be strictly positive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
    /// use orx_iterable::Collection;
    ///
    /// let (num_threads, chunk_size) = (4, 2);
    /// let characters = vec!['0', '1', '2', '3', '4', '5', '6', '7'];
    ///
    /// let iter = characters.con_iter();
    /// let bag = ConcurrentBag::new();
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             iter.for_each(chunk_size, |c| {
    ///                 bag.push(c.to_digit(10).unwrap());
    ///             });
    ///         });
    ///     }
    /// });
    ///
    /// // parent concurrent iterator is completely consumed
    /// assert!(iter.next().is_none());
    ///
    /// let sum: u32 = bag.into_inner().iter().sum();
    /// assert_eq!(sum, (0..8).sum());
    /// ```
    fn for_each_x<Fun: FnMut(Self::Item)>(&self, chunk_size: usize, fun: Fun)
    where
        Self: Sized,
    {
        default_fns::for_each::for_each_x(self, chunk_size, fun);
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
    /// let numbers = vec![3, 4, 1, 9];
    ///
    /// let iter = numbers.con_iter();
    /// let neutral = 0; // neutral for i32 & add
    ///
    /// let sum = std::thread::scope(|s| {
    ///     (0..num_threads)
    ///         .map(|_| s.spawn(|| iter.fold(1, neutral, |x, y| x + y))) // parallel fold
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
        default_fns::fold::fold_x(self, chunk_size, fold, neutral)
    }

    /// Returns Some of the remaining length of the iterator if it is known; returns None otherwise.
    fn try_get_len(&self) -> Option<usize>;

    /// Returns Some of the initial length of the iterator when it was constructed if it is known; returns None otherwise.
    ///
    /// This method is often required and used for certain optimizations in parallel computing.
    fn try_get_initial_len(&self) -> Option<usize>;

    /// Returns whether or not the concurrent iterator has more elements to yield.
    ///
    /// # Examples
    ///
    /// An exact size concurrent iterator will always be certain about this query and will return either `HasMore::No` or `HasMore::Yes(n)` where `n` is the number of remaining elements.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let vec: Vec<_> = (0..512).collect();
    /// let iter = vec.into_con_iter();
    ///
    /// assert_eq!(iter.has_more(), HasMore::Yes(512));
    ///
    /// for _ in 0..500 {
    ///     _ = iter.next();
    /// }
    ///
    /// assert_eq!(iter.has_more(), HasMore::Yes(12));
    ///
    /// while let Some(_) = iter.next() {}
    ///
    /// assert_eq!(iter.has_more(), HasMore::No);
    /// ```
    ///
    /// An non-exact-size concurrent iterator will return:
    /// * either `HasMore::Maybe` when there are elements in the data source to check, or work to do, but not guaranteed that they will be yielded,
    /// * or `HasMore::No` when the iterator has terminated.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let regular_iter = (0..512).map(|x| x + 1).filter(|x| x % 2 == 1);
    /// let iter = regular_iter.into_con_iter();
    ///
    /// assert_eq!(iter.has_more(), HasMore::Maybe);
    ///
    /// for _ in 0..250 {
    ///     _ = iter.next();
    /// }
    ///
    /// assert_eq!(iter.has_more(), HasMore::Maybe);
    ///
    /// while let Some(_) = iter.next() {}
    ///
    /// assert_eq!(iter.has_more(), HasMore::No);
    /// ```
    #[inline(always)]
    fn has_more(&self) -> HasMore {
        match self.try_get_len() {
            None => HasMore::Maybe,
            Some(0) => HasMore::No,
            Some(n) => HasMore::Yes(n),
        }
    }

    /// Creates an iterator which pulls elements from this iterator as chunks of the given `chunk_size`.
    ///
    /// See [`buffered_iter`] for a version that additionally provides the input order of the elements in the chunk.
    ///
    /// [`buffered_iter`]: crate::ConcurrentIter::buffered_iter
    ///
    /// Returned iterator is a regular iterator, except that it is linked to the concurrent iterator and pulls its elements concurrently from the parent iterator.
    /// The `next_x` call of the buffered iterator returns `None` if the concurrent iterator is consumed.
    /// Otherwise, it returns a  non-empty `ExactSizeIterator` containing at least 1 and at most `chunk_size` consecutive elements pulled from the original source data.
    ///
    /// Iteration in chunks is allocation free whenever possible; for instance, when the source data is a vector, a slice or an array, etc. with known size.
    /// On the other hand, when the source data is an arbitrary `Iterator`, iteration in chunks requires a buffer to write the chunk of pulled elements.
    ///
    /// This method is memory efficient in these situations.
    /// It allocates a buffer of `chunk_size` only once when created, only if the source data requires it.
    /// Often, one allocation is required per thread.
    /// This buffer is used over and over until the concurrent iterator is consumed.
    ///
    /// # Panics
    ///
    /// Panics if `chunk_size` is zero; chunk size is required to be strictly positive.
    ///
    /// # Example
    ///
    /// Example below illustrates a parallel sum operation.
    /// Entire iteration requires an allocation (4 threads) * (16 chunk size) = 64 elements.
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let (num_threads, chunk_size) = (4, 64);
    /// let iter = (0..16384).map(|x| x * 2).into_con_iter();
    ///
    /// let sum = std::thread::scope(|s| {
    ///     (0..num_threads)
    ///         .map(|_| {
    ///             s.spawn(|| {
    ///                 let mut sum = 0;
    ///                 let mut buffered = iter.buffered_iter_x(chunk_size);
    ///                 while let Some(chunk) = buffered.next_x() {
    ///                     sum += chunk.sum::<usize>();
    ///                 }
    ///                 sum
    ///             })
    ///         })
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
    fn buffered_iter_x(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIterX> {
        BufferedIter::new(Self::BufferedIterX::new(chunk_size), self)
    }
}
