use crate::{
    cloned::ConIterCloned,
    copied::ConIterCopied,
    enumerate::Enumerate,
    pullers::{ChunkPuller, EnumeratedItemPuller, ItemPuller},
};

/// An iterator which can safely be used concurrently by multiple threads.
///
/// This trait can be considered as the *concurrent counterpart* of the [`Iterator`]
/// trait.
///
/// Practically, this means that elements can be pulled using a shared reference,
/// and therefore, it can be conveniently shared among threads.
///
/// # Examples
///
/// ## A. while let loops: next & next_with_idx
///
/// Main method of a concurrent iterator is the [`next`] which is identical to the
/// `Iterator::next` method except that it requires a shared reference.
/// Additionally, [`next_with_idx`] can be used whenever the index of the element
/// is also required.
///
/// [`next`]: crate::ConcurrentIter::next
/// [`next_with_idx`]: crate::ConcurrentIter::next_with_idx
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec!['x', 'y'];
/// let con_iter = vec.con_iter();
/// assert_eq!(con_iter.next(), Some(&'x'));
/// assert_eq!(con_iter.next_with_idx(), Some((1, &'y')));
/// assert_eq!(con_iter.next(), None);
/// assert_eq!(con_iter.next_with_idx(), None);
/// ```
///
/// This iteration methods yielding optional elements can be used conveniently with
/// `while let` loops.
///
/// In the following program 100 strings in the vector will be processed concurrently
/// by four threads. Note that this is a very convenient but effective way to share
/// tasks among threads especially in heterogeneous scenarios. Every time a thread
/// completes processing a value, it will pull a new element (task) from the iterator.
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let num_threads = 4;
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
/// let con_iter = data.con_iter();
///
/// let process = |_x: &String| { /* assume actual work */ };
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             // concurrently iterate over values in a `while let` loop
///             while let Some(value) = con_iter.next() {
///                 process(value);
///             }
///         });
///     }
/// });
/// ```
///
/// ## B. for loops: item_puller
///
/// Although `while let` loops are considerably convenient, a concurrent iterator
/// cannot be directly used with `for` loops. However, it is possible to create a
/// regular Iterator from a concurrent iterator within a thread which can safely
/// **pull** elements from the concurrent iterator. Since it is a regular Iterator,
/// it can be used with a `for` loop.
///
/// The regular Iterator; i.e., the puller can be created using the [`item_puller`]
/// method. Alternatively, [`item_puller_with_idx`] can be used to create an iterator
/// which also yields the indices of the items.
///
/// Therefore, the parallel processing example above can equivalently implemented
/// as follows.
///
/// [`item_puller`]: crate::ConcurrentIter::item_puller
/// [`item_puller_with_idx`]: crate::ConcurrentIter::item_puller_with_idx
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let num_threads = 4;
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
/// let con_iter = data.con_iter();
///
/// let process = |_x: &String| { /* assume actual work */ };
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             // concurrently iterate over values in a `for` loop
///             for value in con_iter.item_puller() {
///                 process(value);
///             }
///         });
///     }
/// });
/// ```
///
/// It is important to emphasize that the [`ItemPuller`] implements a regular [`Iterator`].
/// This not only enables the `for` loops but also makes all iterator methods available.
/// For instance, we can use `filter`, `map` and/or `reduce` on the item puller iterator
/// as we do with regular iterators, while under the hood it will concurrently iterate
/// over the elements of the concurrent iterator.
///
/// [`ItemPuller`]: crate::ItemPuller
///
/// ## C. Iteration by Chunks
///
/// Iteration using `next`, `next_with_idx` or via the pullers created by `item_puller`
/// or `item_puller_with_idx` all pull elements from the data source one by one.
/// This is exactly similar to iteration by a regular Iterator. However, depending on the
/// use case, this is not always what we want in a concurrent program.
///
/// Due to the following reason.
///
/// Concurrent iterators use atomic variables which have an overhead compared to sequential
/// iterators. Every time we pull an element from a concurrent iterator, its atomic state is
/// updated. Therefore, the fewer times we update the atomic state, the less significant the
/// overhead. The way to achieve fewer updates is through pulling multiple elements at once,
/// rather than one element at a time.
/// * Note that this can be considered as an optimization technique which might or might
///   not be relevant. The rule of thumb is as follows; the more work we do on each element
///   (or equivalently, the larger the `process` is), the less significant the overhead is.
///
/// Nevertheless, it is conveniently possible to achieve fewer updates using chunk pullers.
/// A chunk puller is similar to the item puller except that it pulls multiple elements at
/// once. A chunk puller can be created from a concurrent iterator using the [`chunk_puller`]
/// method.
///
/// The following program uses a chunk puller. Chunk puller's [`pull`] method returns an option
/// of an [`ExactSizeIterator`]. The `ExactSizeIterator` will contain 10 elements, or less if
/// not left enough, but never 0 elements (in this case `pull` returns None). This allows for
/// using a `while let` loop. Then, we can iterate over the `chunk` which is a regular iterator.
///
/// Note that, we can also use [`pull_with_idx`] whenever the indices are also required.
///
/// [`chunk_puller`]: crate::ConcurrentIter::chunk_puller
/// [`pull`]: crate::ChunkPuller::pull
/// [`pull_with_idx`]: crate::ChunkPuller::pull_with_idx
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
/// ## D. Iteration by Flattened Chunks
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
/// It is important to note that, when we say we pull 10 items, we actually only reserve these
/// elements for the corresponding thread. We do not actually clone elements or copy memory.
///
/// ## E. Early Exit
///
/// Concurrent iterators also support early exit scenarios through a simple method call,
/// [`skip_to_end`]. Whenever, any of the threads observes a certain condition and decides that
/// it is no longer necessary to iterate over the remaining elements, it can call `skip_to_end`.
///
/// Threads approaching the concurrent iterator to pull more elements after this call will
/// observe that there are no other elements left and may exit.
///
/// One common use case is the `find` method of iterators. The following is a parallel implementation
/// of `find` using concurrent iterators.
///
/// In the following program, one of the threads will find "33" satisfying the predicate and will call
/// `skip_to_end` to jump to end of the iterator. In the example setting, it is possible that other threads
/// might still process some more items:
///
/// * Just while the thread that found "33" is evaluating the predicate, other threads might pull a
///   few more items, say 34, 35 and 36.
/// * While they might be comparing these items against the predicate, the winner thread calls `skip_to_end`.
/// * After this point, the item pullers' next calls will all return None.
/// * This will allow all threads to return & join, without actually going through all 1000 elements of the
///   data source.
///
/// In this regard, `skip_to_end` allows for a little communication among threads in early exit scenarios.
///
/// [`skip_to_end`]: crate::ConcurrentIter::skip_to_end
///
/// ```
/// use orx_concurrent_iter::*;
///
/// fn find<T, F>(
///     num_threads: usize,
///     con_iter: impl ConcurrentIter<Item = T>,
///     predicate: F,
/// ) -> Option<T>
/// where
///     T: Send + Sync,
///     F: Fn(&T) -> bool + Send + Sync,
/// {
///     std::thread::scope(|s| {
///         let mut results = vec![];
///         for _ in 0..num_threads {
///             results.push(s.spawn(|| {
///                 for value in con_iter.item_puller() {
///                     if predicate(&value) {
///                         // will immediately jump to end
///                         con_iter.skip_to_end();
///                         return Some(value);
///                     }
///                 }
///                 None
///             }));
///         }
///         results.into_iter().filter_map(|x| x.join().unwrap()).next()
///     })
/// }
///
/// let data: Vec<_> = (0..1000).map(|x| x.to_string()).collect();
/// let value = find(4, data.con_iter(), |x| x.starts_with("33"));
///
/// assert_eq!(value, Some(&33.to_string()));
/// ```
///
/// ## F. Back to Sequential Iterator
///
/// Every concurrent iterator can be consumed and converted into a regular sequential
/// iterator using [`into_seq_iter`] method. In this sense, it can be considered as a
/// generalization of iterators that can be iterated over either concurrently or sequentially.
///
/// [`into_seq_iter`]: crate::ConcurrentIter::into_seq_iter
pub trait ConcurrentIter: Send + Sync {
    /// Type of the element that the concurrent iterator yields.
    type Item: Send + Sync;

    /// Type of the sequential iterator that the concurrent iterator can be converted
    /// into using the [`into_seq_iter`] method.
    ///
    /// [`into_seq_iter`]: crate::ConcurrentIter::into_seq_iter
    type SequentialIter: Iterator<Item = Self::Item>;

    /// Type of the chunk puller that can be created using the [`chunk_puller`] method.
    ///
    /// [`chunk_puller`]: crate::ConcurrentIter::chunk_puller
    type ChunkPuller<'i>: ChunkPuller<ChunkItem = Self::Item>
    where
        Self: 'i;

    // transform

    /// Converts the concurrent iterator into its sequential regular counterpart.
    /// Note that the sequential iterator is a regular [`Iterator`], and hence,
    /// does not have any overhead related with atomic states. Therefore, it is
    /// useful where the program decides to iterate over a single thread rather
    /// than concurrently by multiple threads.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let data = vec!['x', 'y'];
    ///
    /// // con_iter implements ConcurrentIter
    /// let con_iter = data.into_con_iter();
    ///
    /// // seq_iter implements regular Iterator
    /// // it has the same type as the iterator we would
    /// // have got with `data.into_iter()`
    /// let mut seq_iter = con_iter.into_seq_iter();
    /// assert_eq!(seq_iter.next(), Some('x'));
    /// assert_eq!(seq_iter.next(), Some('y'));
    /// assert_eq!(seq_iter.next(), None);
    /// ```
    fn into_seq_iter(self) -> Self::SequentialIter;

    // iterate

    /// Immediately jumps to the end of the iterator, skipping the remaining elements.
    ///
    /// This method is useful in early-exit scenarios which allows not only the thread
    /// calling this method to return early, but also all other threads that are iterating
    /// over this concurrent iterator to return early since they would not find any more
    /// remaining elements.
    ///
    /// # Example
    ///
    /// One common use case is the `find` method of iterators. The following is a parallel implementation
    /// of `find` using concurrent iterators.
    ///
    /// In the following program, one of the threads will find "33" satisfying the predicate and will call
    /// `skip_to_end` to jump to end of the iterator. In the example setting, it is possible that other threads
    /// might still process some more items:
    ///
    /// * Just while the thread that found "33" is evaluating the predicate, other threads might pull a
    ///   few more items, say 34, 35 and 36.
    /// * While they might be comparing these items against the predicate, the winner thread calls `skip_to_end`.
    /// * After this point, the item pullers' next calls will all return None.
    /// * This will allow all threads to return & join, without actually going through all 1000 elements of the
    ///   data source.
    ///
    /// In this regard, `skip_to_end` allows for a little communication among threads in early exit scenarios.
    ///
    /// [`skip_to_end`]: crate::ConcurrentIter::skip_to_end
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// fn find<T, F>(
    ///     num_threads: usize,
    ///     con_iter: impl ConcurrentIter<Item = T>,
    ///     predicate: F,
    /// ) -> Option<T>
    /// where
    ///     T: Send + Sync,
    ///     F: Fn(&T) -> bool + Send + Sync,
    /// {
    ///     std::thread::scope(|s| {
    ///         let mut results = vec![];
    ///         for _ in 0..num_threads {
    ///             results.push(s.spawn(|| {
    ///                 for value in con_iter.item_puller() {
    ///                     if predicate(&value) {
    ///                         // will immediately jump to end
    ///                         con_iter.skip_to_end();
    ///                         return Some(value);
    ///                     }
    ///                 }
    ///                 None
    ///             }));
    ///         }
    ///         results.into_iter().filter_map(|x| x.join().unwrap()).next()
    ///     })
    /// }
    ///
    /// let data: Vec<_> = (0..1000).map(|x| x.to_string()).collect();
    /// let value = find(4, data.con_iter(), |x| x.starts_with("33"));
    ///
    /// assert_eq!(value, Some(&33.to_string()));
    /// ```
    fn skip_to_end(&self);

    /// Returns the next element of the iterator.
    /// It returns None if there are no more elements left.
    ///
    /// Notice that this method requires a shared reference rather than a mutable reference, and hence,
    /// can be called concurrently from multiple threads.
    ///
    /// See also [`next_with_idx`] in order to receive additionally the index of the elements.
    ///
    /// [`next_with_idx`]: crate::ConcurrentIter::next_with_idx
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec!['x', 'y'];
    /// let con_iter = vec.con_iter();
    /// assert_eq!(con_iter.next(), Some(&'x'));
    /// assert_eq!(con_iter.next(), Some(&'y'));
    /// assert_eq!(con_iter.next(), None);
    /// ```
    ///
    /// This iteration methods yielding optional elements can be used conveniently with
    /// `while let` loops.
    ///
    /// In the following program 100 strings in the vector will be processed concurrently
    /// by four threads. Note that this is a very convenient but effective way to share
    /// tasks among threads especially in heterogeneous scenarios. Every time a thread
    /// completes processing a value, it will pull a new element (task) from the iterator.
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 4;
    /// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
    /// let con_iter = data.con_iter();
    ///
    /// let process = |_x: &String| { /* assume actual work */ };
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             // concurrently iterate over values in a `while let` loop
    ///             while let Some(value) = con_iter.next() {
    ///                 process(value);
    ///             }
    ///         });
    ///     }
    /// });
    /// ```
    fn next(&self) -> Option<Self::Item>;

    /// Returns the next element of the iterator together its index.
    /// It returns None if there are no more elements left.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec!['x', 'y'];
    /// let con_iter = vec.con_iter();
    /// assert_eq!(con_iter.next_with_idx(), Some((0, &'x')));
    /// assert_eq!(con_iter.next_with_idx(), Some((1, &'y')));
    /// assert_eq!(con_iter.next_with_idx(), None);
    /// ```
    fn next_with_idx(&self) -> Option<(usize, Self::Item)>;

    // len

    /// Returns the bounds on the remaining length of the iterator.
    ///
    /// The first element is the lower bound, and the second element is the upper bound.
    ///
    /// Having an upper bound of None means that there is no knowledge of a limit of the number of
    /// remaining elements.
    ///
    /// Having a tuple of `(x, Some(x))` means that, we are certain about the number of remaining
    /// elements, which `x`. When the concurrent iterator additionally implements [`ExactSizeConcurrentIter`],
    /// then its `len` method also returns `x`.
    ///
    /// [`ExactSizeConcurrentIter`]: crate::ExactSizeConcurrentIter
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// // implements ExactSizeConcurrentIter
    ///
    /// let data = vec!['x', 'y', 'z'];
    /// let con_iter = data.con_iter();
    /// assert_eq!(con_iter.size_hint(), (3, Some(3)));
    /// assert_eq!(con_iter.len(), 3);
    ///
    /// assert_eq!(con_iter.next(), Some(&'x'));
    /// assert_eq!(con_iter.size_hint(), (2, Some(2)));
    /// assert_eq!(con_iter.len(), 2);
    ///
    /// // does not implement ExactSizeConcurrentIter
    ///
    /// let iter = data.iter().filter(|x| **x != 'y');
    /// let con_iter = iter.iter_into_con_iter();
    /// assert_eq!(con_iter.size_hint(), (0, Some(3)));
    ///
    /// assert_eq!(con_iter.next(), Some(&'x'));
    /// assert_eq!(con_iter.size_hint(), (0, Some(2)));
    ///
    /// assert_eq!(con_iter.next(), Some(&'z'));
    /// assert_eq!(con_iter.size_hint(), (0, Some(0)));
    /// ```
    fn size_hint(&self) -> (usize, Option<usize>);

    /// Returns `Some(x)` if the number of remaining items is known with certainly and if it
    /// is equal to `x`.
    ///
    /// It returns None otherwise.
    ///
    /// Note that this is a shorthand for:
    ///
    /// ```ignore
    /// match con_iter.size_hint() {
    ///     (x, Some(y)) if x == y => Some(x),
    ///     _ => None,
    /// }
    /// ```
    fn try_get_len(&self) -> Option<usize> {
        match self.size_hint() {
            (x, Some(y)) if x == y => Some(x),
            _ => None,
        }
    }

    // pullers

    /// Creates a [`ChunkPuller`] from the concurrent iterator.
    /// The created chunk puller can be used to [`pull`] `chunk_size` elements at once from the
    /// data source, rather than pulling one by one.
    ///
    /// Iterating over chunks using a chunk puller rather than single elements is an optimization
    /// technique. Chunk pullers enable a convenient way to apply this optimization technique
    /// which is not relevant for certain scenarios, while it is very effective for others.
    ///
    /// The reason why we would want to iterate over chunks is as follows.
    ///
    /// Concurrent iterators use atomic variables which have an overhead compared to sequential
    /// iterators. Every time we pull an element from a concurrent iterator, its atomic state is
    /// updated. Therefore, the fewer times we update the atomic state, the less significant the
    /// overhead. The way to achieve fewer updates is through pulling multiple elements at once,
    /// rather than one element at a time.
    /// * The more work we do on each element, the less significant the overhead is.
    ///
    /// Nevertheless, it is conveniently possible to achieve fewer updates using chunk pullers.
    /// A chunk puller is similar to the item puller except that it pulls multiple elements at
    /// once.
    ///
    /// The following program uses a chunk puller. Chunk puller's [`pull`] method returns an option
    /// of an [`ExactSizeIterator`]. The `ExactSizeIterator` will contain 10 elements, or less if
    /// not left enough, but never 0 elements (in this case `pull` returns None). This allows for
    /// using a `while let` loop. Then, we can iterate over the `chunk` which is a regular iterator.
    ///
    /// Note that, we can also use [`pull_with_idx`] whenever the indices are also required.
    ///
    /// [`chunk_puller`]: crate::ConcurrentIter::chunk_puller
    /// [`pull`]: crate::ChunkPuller::pull
    /// [`pull_with_idx`]: crate::ChunkPuller::pull_with_idx
    /// [`ChunkPuller`]: crate::ChunkPuller
    /// [`pull`]: crate::ChunkPuller::pull
    ///
    /// # Examples
    ///
    /// ## Iteration by Chunks
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
    /// ## Iteration by Flattened Chunks
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
    /// It is important to note that, when we say we pull 10 items, we actually only reserve these
    /// elements for the corresponding thread. We do not actually clone elements or copy memory.
    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_>;

    /// Creates a [`ItemPuller`] from the concurrent iterator.
    /// The created item puller can be used to pull elements one by one from the
    /// data source.
    ///
    /// Note that `ItemPuller` implements a regular [`Iterator`].
    /// This not only enables the `for` loops but also makes all iterator methods available.
    /// For instance, we can use `filter`, `map` and/or `reduce` on the item puller iterator
    /// as we do with regular iterators, while under the hood it will concurrently iterate
    /// over the elements of the concurrent iterator.
    ///
    /// Alternatively, [`item_puller_with_idx`] can be used to create an iterator
    /// which also yields the indices of the items.
    ///
    /// [`item_puller`]: crate::ConcurrentIter::item_puller
    /// [`item_puller_with_idx`]: crate::ConcurrentIter::item_puller_with_idx
    ///
    /// # Examples
    ///
    /// ## Concurrent looping with `for`
    ///
    /// In the following program, we use a regular `for` loop over the item pullers, one created
    /// created for each thread. All item pullers being created from the same concurrent iterator
    /// will actually concurrently pull items from the same data source.
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 4;
    /// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
    /// let con_iter = data.con_iter();
    ///
    /// let process = |_x: &String| { /* assume actual work */ };
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             // concurrently iterate over values in a `for` loop
    ///             for value in con_iter.item_puller() {
    ///                 process(value);
    ///             }
    ///         });
    ///     }
    /// });
    /// ```
    ///
    /// ## Parallel reduce
    ///
    /// As mentioned above, item puller makes all convenient Iterator methods available in a concurrent
    /// program. The following simple program demonstrate a very convenient way to implement a parallel
    /// reduce operation:
    ///
    /// * Within each thread, we reduce the elements that are pulled by the thread by summing up the
    ///   number of characters of each item.
    /// * The sums by each thread are collected in the `thread_sums` vector.
    /// * Finally, we sum the `threads_sums` to obtain the `global_sum`.
    /// * The assertion verifies that the result of this parallel reduction using 4 threads is equal
    ///   to the expected result that is computed by reduction using a single thread.
    ///
    /// This parallel program could have been achieved by different approaches. However, the objective
    /// and convenience of the item pullers is using the same computation both concurrently and
    /// sequentially; i.e., `.map(|x| x.len()).sum()`.
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let num_threads = 4;
    /// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
    /// let con_iter = data.con_iter();
    ///
    /// let global_sum: usize = std::thread::scope(|s| {
    ///     let mut thread_sums = vec![];
    ///     for _ in 0..num_threads {
    ///         thread_sums.push(s.spawn(|| {
    ///             // sum of number of characters of items pulled by this thread
    ///             con_iter.item_puller().map(|x| x.len()).sum::<usize>()
    ///         }));
    ///     }
    ///     thread_sums.into_iter().map(|x| x.join().unwrap()).sum()
    /// });
    ///
    /// assert_eq!(global_sum, data.iter().map(|x| x.len()).sum());
    /// ```
    fn item_puller(&self) -> ItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    /// Creates a [`EnumeratedItemPuller`] from the concurrent iterator.
    /// The created item puller can be used to `pull` elements one by one from the
    /// data source together with the index of the elements.
    ///
    /// Note that `EnumeratedItemPuller` implements a regular [`Iterator`].
    /// This not only enables the `for` loops but also makes all iterator methods available.
    /// For instance, we can use `filter`, `map` and/or `reduce` on the item puller iterator
    /// as we do with regular iterators, while under the hood it will concurrently iterate
    /// over the elements of the concurrent iterator.
    ///
    /// [`EnumeratedItemPuller`]: crate::EnumeratedItemPuller
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
    /// let process = |_idx: usize, _x: &String| { /* assume actual work */ };
    ///
    /// std::thread::scope(|s| {
    ///     for _ in 0..num_threads {
    ///         s.spawn(|| {
    ///             // concurrently iterate over values in a `for` loop
    ///             for (idx, value) in con_iter.item_puller_with_idx() {
    ///                 process(idx, value);
    ///             }
    ///         });
    ///     }
    /// });
    /// ```
    fn item_puller_with_idx(&self) -> EnumeratedItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    // provided transformations

    /// Creates an iterator which copies all of its elements.
    ///
    /// This is useful when you have an iterator over `&T`, but you need an iterator over `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec!['x', 'y'];
    ///
    /// let con_iter = vec.con_iter();
    /// assert_eq!(con_iter.next(), Some(&'x'));
    /// assert_eq!(con_iter.next(), Some(&'y'));
    /// assert_eq!(con_iter.next(), None);
    ///
    /// let con_iter = vec.con_iter().copied();
    /// assert_eq!(con_iter.next(), Some('x'));
    /// assert_eq!(con_iter.next(), Some('y'));
    /// assert_eq!(con_iter.next(), None);
    /// ```
    fn copied<'a, T>(self) -> ConIterCopied<'a, Self, T>
    where
        T: Send + Sync + Copy,
        Self: ConcurrentIter<Item = &'a T> + Sized,
    {
        ConIterCopied::new(self)
    }

    /// Creates an iterator which clones all of its elements.
    ///
    /// This is useful when you have an iterator over `&T`, but you need an iterator over `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec![String::from("x"), String::from("y")];
    ///
    /// let con_iter = vec.con_iter();
    /// assert_eq!(con_iter.next(), Some(&String::from("x")));
    /// assert_eq!(con_iter.next(), Some(&String::from("y")));
    /// assert_eq!(con_iter.next(), None);
    ///
    /// let con_iter = vec.con_iter().cloned();
    /// assert_eq!(con_iter.next(), Some(String::from("x")));
    /// assert_eq!(con_iter.next(), Some(String::from("y")));
    /// assert_eq!(con_iter.next(), None);
    /// ```
    fn cloned<'a, T>(self) -> ConIterCloned<'a, Self, T>
    where
        T: Send + Sync + Clone,
        Self: ConcurrentIter<Item = &'a T> + Sized,
    {
        ConIterCloned::new(self)
    }

    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }
}

#[test]
fn abc() {
    use crate::*;

    let vec = vec!['x', 'y'];

    let con_iter = vec.con_iter();
    assert_eq!(con_iter.next(), Some(&'x'));
    assert_eq!(con_iter.next(), Some(&'y'));
    assert_eq!(con_iter.next(), None);

    let con_iter = vec.con_iter().copied();
    assert_eq!(con_iter.next(), Some('x'));
    assert_eq!(con_iter.next(), Some('y'));
    assert_eq!(con_iter.next(), None);
}
