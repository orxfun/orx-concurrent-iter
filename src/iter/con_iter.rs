use super::{
    buffered::{
        buffered_chunk::{BufferedChunk, BufferedChunkX},
        buffered_iter::BufferedIter,
    },
    default_fns,
};
use crate::{
    next::{Next, NextChunk},
    ConIterIdsAndValues, ConcurrentIterX,
};

/// Trait defining a concurrent iterator with `next` and `next_id_and_chunk` methods which can safely be called my multiple threads concurrently.
pub trait ConcurrentIter: ConcurrentIterX {
    /// Type of the buffered iterator returned by the `chunk_iter` method when elements are fetched in chunks by each thread.
    type BufferedIter: BufferedChunk<Self::Item, ConIter = Self>;

    #[inline(always)]
    fn into_concurrent_iter_x(self) -> impl ConcurrentIterX<Item = Self::Item>
    where
        Self: Sized,
    {
        self
    }

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
    /// fn to_str(num: usize) -> String {
    ///     num.to_string()
    /// }
    ///
    /// let (num_threads, chunk_size) = (4, 32);
    /// let strings: Vec<_> = (0..1024).map(to_str).collect();
    /// let bag = ConcurrentBag::new();
    ///
    /// let iter = strings.con_iter();
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             while let Some(next) = iter.next_id_and_value() {
    ///                 // idx is the original position in `characters`
    ///                 assert_eq!(next.value, &to_str(next.idx));
    ///                 bag.push((next.idx, next.value.len()));
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut outputs: Vec<_> = bag.into_inner().into();
    /// outputs.sort_by_key(|x| x.0); // sort to original order
    /// for (x, y) in outputs.iter().map(|x| x.1).zip(&strings) {
    ///     assert_eq!(x, y.len());
    /// }
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
    /// fn to_str(num: usize) -> String {
    ///     num.to_string()
    /// }
    ///
    /// let (num_threads, chunk_size) = (4, 32);
    /// let strings: Vec<_> = (0..1024).map(to_str).collect();
    /// let bag = ConcurrentBag::new();
    ///
    /// let iter = strings.con_iter();
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             while let Some(chunk) = iter.next_chunk(chunk_size) {
    ///                 for (i, value) in chunk.values.enumerate() {
    ///                     // idx is the original position in `characters`
    ///                     let idx = chunk.begin_idx + i;
    ///                     assert_eq!(value, &to_str(idx));
    ///                     bag.push((idx, value.len()));
    ///                 }
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut outputs: Vec<_> = bag.into_inner().into();
    /// outputs.sort_by_key(|x| x.0); // sort to original order
    /// for (x, y) in outputs.iter().map(|x| x.1).zip(&strings) {
    ///     assert_eq!(x, y.len());
    /// }
    /// ```
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>>;

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
    ///             for (idx, value) in iter.ids_and_values() {
    ///                 // idx is the original position in `characters`
    ///                 assert_eq!(value, &to_str(idx));
    ///                 bag.push((idx, value.len()));
    ///             }
    ///         });
    ///     }
    /// });
    ///
    /// let mut outputs: Vec<_> = bag.into_inner().into();
    /// outputs.sort_by_key(|x| x.0); // sort to original order
    /// for (x, y) in outputs.iter().map(|x| x.1).zip(&strings) {
    ///     assert_eq!(x, y.len());
    /// }
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
    /// # Panics
    ///
    /// Panics if `chunk_size` is zero; chunk size is required to be strictly positive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
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
    /// # Panics
    ///
    /// Panics if `chunk_size` is zero; chunk size is required to be strictly positive.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    /// use orx_concurrent_bag::*;
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
    ///             iter.enumerate_for_each(chunk_size, |i, c| {
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
    /// assert!(iter.next().is_none());
    ///
    /// let sum: u32 = bag.into_inner().iter().sum();
    /// assert_eq!(sum, (0..8).sum());
    /// ```
    fn enumerate_for_each<Fun: FnMut(usize, Self::Item)>(&self, chunk_size: usize, fun: Fun)
    where
        Self: Sized,
    {
        default_fns::for_each::for_each_with_ids(self, chunk_size, fun)
    }

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
    ///                 let mut buffered = iter.buffered_iter(chunk_size);
    ///                 while let Some(chunk) = buffered.next() {
    ///                     sum += chunk.values.sum::<usize>();
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
    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        BufferedIter::new(Self::BufferedIter::new(chunk_size), self)
    }
}
