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

    /// Applies the function `f` to each element of the iterator concurrently.
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
    fn for_each_n<F: FnMut(Self::Item)>(&self, chunk_size: usize, f: F);

    /// Applies the function `f` to each index and corresponding element of the iterator concurrently.
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
    fn enumerate_for_each_n<F: FnMut(usize, Self::Item)>(&self, chunk_size: usize, f: F);

    /// Applies the function `f` to each element of the iterator concurrently.
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
    fn for_each<F: FnMut(Self::Item)>(&self, f: F) {
        self.for_each_n(1, f)
    }

    /// Applies the function `f` to each index and corresponding element of the iterator concurrently.
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
    fn enumerate_for_each<F: FnMut(usize, Self::Item)>(&self, f: F) {
        self.enumerate_for_each_n(1, f)
    }
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
