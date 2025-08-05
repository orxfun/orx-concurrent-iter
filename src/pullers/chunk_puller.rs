use crate::pullers::{FlattenedChunkPuller, FlattenedEnumeratedChunkPuller};

/// A chunk puller which is created from and linked to and pulls its elements
/// from a [`ConcurrentIter`].
///
/// It can be created using the [`chunk_puller`] method of a concurrent iterator
/// by providing a desired chunk size.
///
/// [`chunk_puller`]: crate::ConcurrentIter::chunk_puller
///
/// Unlike the [`ItemPuller`], a chunk puller pulls many items at once:
///
/// * Its [`pull`] method pulls a chunk from the concurrent iterator, where:
///   * the pulled chunk implements [`ExactSizeIterator`],
///   * it often has `chunk_size` elements as long as there are sufficient
///     items; less items will be pulled only when the concurrent iterator
///     runs out of elements,
///   * it has at least 1 element, as `pull` returns None if there are no
///     items left.
///
/// Three points are important:
///
/// * Items in each pulled chunk are guaranteed to be sequential in the data
///   source.
/// * Pulling elements in chunks rather than one-by-one as by the `ItemPuller` is
///   an optimization technique which aims to reduce the overhead of updating the
///   atomic state of the concurrent iterator. This optimization is relevant for
///   cases where the work done on the pulled elements are considerably small.
/// * Pulling multiple elements or a chunk does not mean the elements are copied
///   and stored elsewhere. It actually means reserving multiple elements at once
///   for the pulling thread.
///
/// [`ItemPuller`]: crate::ItemPuller
/// [`pull`]: crate::ChunkPuller::pull
/// [`ConcurrentIter`]: crate::ConcurrentIter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let num_threads = 4;
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
/// let con_iter = data.con_iter();
///
/// let process = |_x: &String| {};
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             // concurrently iterate over values in a `while let` loop
///             // while pulling (up to) 10 elements every time
///             let mut chunk_puller = con_iter.chunk_puller(10);
///             while let Some(chunk) = chunk_puller.pull() {
///                 // chunk is an ExactSizeIterator
///                 for value in chunk {
///                     process(value);
///                 }
///             }
///         });
///     }
/// });
/// ```
///
/// The above code conveniently allows for the iteration-by-chunks optimization.
/// However, you might have noticed that now we have a nested `while let` and `for` loops.
/// In terms of convenience, we can do better than this without losing any performance.
///
/// This can be achieved using the [`flattened`] method of the chunk puller (see also
/// [`flattened_with_idx`]).
///
/// [`flattened`]: crate::ChunkPuller::flattened
/// [`flattened_with_idx`]: crate::ChunkPuller::flattened_with_idx
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let num_threads = 4;
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
/// let con_iter = data.con_iter();
///
/// let process = |_x: &String| {};
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             // concurrently iterate over values in a `for` loop
///             // while concurrently pulling (up to) 10 elements every time
///             for value in con_iter.chunk_puller(10).flattened() {
///                 process(value);
///             }
///         });
///     }
/// });
/// ```
///
/// A bit of magic here, that requires to be explained below.
///
/// Notice that this is a very convenient way to concurrently iterate over the elements
/// using a simple `for` loop. However, it is important to note that, under the hood, this is
/// equivalent to the program in the previous section where we used the `pull` method of the
/// chunk puller.
///
/// The following happens under the hood:
///
/// * We reach the concurrent iterator to pull 10 items at once from the data source.
///   This is the intended performance optimization to reduce the updates of the atomic state.
/// * Then, we iterate one-by-one over the pulled 10 items inside the thread as a regular iterator.
/// * Once, we complete processing these 10 items, we approach the concurrent iterator again.
///   Provided that there are elements left, we pull another chunk of 10 items.
/// * Then, we iterate one-by-one ...
///
/// See the [`ItemPuller`] documentation for the notes on how the pullers bring the convenience of
/// Iterator methods to concurrent programs, which is demonstrated by a 4-line implementation of the
/// parallelized [`reduce`]. We can add the iteration-by-chunks optimization on top of this while
/// keeping the implementation as simple and fitting 4-lines due to the fact that flattened chunk
/// puller implements Iterator.
///
/// In the following code, the sums are computed by 8 threads while each thread pulls elements in
/// chunks of 64.
///
/// ```
/// use orx_concurrent_iter::*;
///
/// fn parallel_reduce<T, F>(
///     num_threads: usize,
///     chunk: usize,
///     con_iter: impl ConcurrentIter<Item = T>,
///     reduce: F,
/// ) -> Option<T>
/// where
///     T: Send,
///     F: Fn(T, T) -> T + Sync,
/// {
///     std::thread::scope(|s| {
///         (0..num_threads)
///             .map(|_| s.spawn(|| con_iter.chunk_puller(chunk).flattened().reduce(&reduce))) // reduce inside each thread
///             .filter_map(|x| x.join().unwrap()) // join threads, ignore None's
///             .reduce(&reduce) // reduce thread results to final result
///     })
/// }
///
/// let n = 10_000;
/// let data: Vec<_> = (0..n).collect();
/// let sum = parallel_reduce(8, 64, data.con_iter().copied(), |a, b| a + b);
/// assert_eq!(sum, Some(n * (n - 1) / 2));
/// ```
///
/// [`reduce`]: Iterator::reduce
/// [`ItemPuller`]: crate::ItemPuller
pub trait ChunkPuller {
    /// Type of the element that the concurrent iterator yields.
    type ChunkItem;

    /// Type of the pulled chunks which implements [`ExactSizeIterator`].
    type Chunk<'c>: ExactSizeIterator<Item = Self::ChunkItem> + Default
    where
        Self: 'c;

    /// Target length of the pulled chunks.
    ///
    /// Note that the pulled chunk might have a length of:
    ///
    /// * `chunk_size` if there are at least `chunk_size` items in the concurrent
    ///   iterator when the `pull` method is called; or
    /// * `n` items where `1 <= n < chunk_size` items if the concurrent iterator
    ///   still has items; however, fewer than `chunk_size`.
    ///
    /// Notice that the chunk cannot contain `0` elements; in which case the `pull`
    /// method returns `None` which signals to complete the iteration. This design
    /// choice allows to use the `while let Some(chunk) = chunk_puller.pull() { }`
    /// loops.
    fn chunk_size(&self) -> usize;

    /// Pulls the next chunk from the connected concurrent iterator.
    ///
    /// The pulled chunk has a known length, and hence, implements [`ExactSizeIterator`].
    /// It might have a length of:
    ///
    /// * `chunk_size` if there are at least `chunk_size` sequential items in the concurrent
    ///   iterator when the `pull` method is called; or
    /// * `n` items where `1 <= n < chunk_size` sequential items if the concurrent iterator
    ///   still has items; however, fewer than `chunk_size`.
    ///
    /// Notice that the chunk cannot contain `0` elements; in which case the `pull`
    /// method returns `None` which signals to complete the iteration. This design
    /// choice allows to use the `while let Some(chunk) = chunk_puller.pull() { }`
    /// loops.
    fn pull(&mut self) -> Option<Self::Chunk<'_>>;

    /// Pulls the next chunk from the connected concurrent iterator together with the index
    /// of the first element of the chunk.
    ///
    /// Since all items in the pulled chunk are sequential, knowing the index of the first
    /// item of the chunk provides the complete information of all indices.
    ///
    /// # Examples
    ///
    /// Performing an enumerated iteration while using chunks is demonstrated below.
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 4;
    /// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
    /// let con_iter = data.con_iter();
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             let mut chunk_puller = con_iter.chunk_puller(4);
    ///             while let Some((begin_idx, chunk)) = chunk_puller.pull_with_idx() {
    ///                 for (i, value) in chunk.enumerate() { // i is the index within chunk
    ///                     let idx = begin_idx + i; // idx is the index in the input collection
    ///                     assert_eq!(value, &idx.to_string());
    ///                 }
    ///             }
    ///         });
    ///     }
    /// });
    /// ```
    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)>;

    /// Converts the [`ChunkPuller`] into a [`FlattenedChunkPuller`] which is still connected to
    /// and pulls its elements from the same concurrent iterator; while allowing for:
    ///
    /// * avoiding nested loops:
    ///   * `while let` loop to [`pull`] the chunk, and then
    ///   * iterate over the chunk;
    /// * bringing Iterator methods to the concurrent programs since [`FlattenedChunkPuller`]
    ///   implements the regular [`Iterator`].
    ///
    /// [`FlattenedChunkPuller`]: crate::FlattenedChunkPuller
    /// [`pull`]: crate::ChunkPuller::pull
    ///
    /// # Examples
    ///
    /// See the [`ItemPuller`] documentation for the notes on how the pullers bring the convenience of
    /// Iterator methods to concurrent programs, which is demonstrated by a 4-line implementation of the
    /// parallelized [`reduce`]. We can add the iteration-by-chunks optimization on top of this while
    /// keeping the implementation as simple and fitting 4-lines due to the fact that flattened chunk
    /// puller implements Iterator.
    ///
    /// In the following code, the sums are computed by 8 threads while each thread pulls elements in
    /// chunks of 64.
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// fn parallel_reduce<T, F>(
    ///     num_threads: usize,
    ///     chunk: usize,
    ///     con_iter: impl ConcurrentIter<Item = T>,
    ///     reduce: F,
    /// ) -> Option<T>
    /// where
    ///     T: Send,
    ///     F: Fn(T, T) -> T + Sync,
    /// {
    ///     std::thread::scope(|s| {
    ///         (0..num_threads)
    ///             .map(|_| s.spawn(|| con_iter.chunk_puller(chunk).flattened().reduce(&reduce))) // reduce inside each thread
    ///             .filter_map(|x| x.join().unwrap()) // join threads, ignore None's
    ///             .reduce(&reduce) // reduce thread results to final result
    ///     })
    /// }
    ///
    /// let n = 10_000;
    /// let data: Vec<_> = (0..n).collect();
    /// let sum = parallel_reduce(8, 64, data.con_iter().copied(), |a, b| a + b);
    /// assert_eq!(sum, Some(n * (n - 1) / 2));
    /// ```
    ///
    /// [`reduce`]: Iterator::reduce
    /// [`ItemPuller`]: crate::ItemPuller
    fn flattened<'c>(self) -> FlattenedChunkPuller<'c, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    /// Converts the [`ChunkPuller`] into a [`FlattenedEnumeratedChunkPuller`] which is still connected to
    /// and pulls its elements from the same concurrent iterator; while allowing for:
    ///
    /// * avoiding nested loops:
    ///   * `while let` loop to [`pull`] the chunk, and then
    ///   * iterate over the chunk;
    /// * bringing Iterator methods to the concurrent programs since [`FlattenedEnumeratedChunkPuller`]
    ///   implements the regular [`Iterator`].
    ///
    /// Similar to [`flattened`] except that returned iterator additionally returns the indices of the
    /// elements in the concurrent iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 4;
    /// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
    /// let con_iter = data.con_iter();
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             for (idx, value) in con_iter.chunk_puller(10).flattened_with_idx() {
    ///                 assert_eq!(value, &idx.to_string());
    ///             }
    ///         });
    ///     }
    /// });
    /// ```
    ///
    /// [`FlattenedEnumeratedChunkPuller`]: crate::FlattenedEnumeratedChunkPuller
    /// [`pull`]: crate::ChunkPuller::pull
    /// [`flattened`]: crate::ChunkPuller::flattened
    fn flattened_with_idx<'c>(self) -> FlattenedEnumeratedChunkPuller<'c, Self>
    where
        Self: Sized,
    {
        self.into()
    }
}
