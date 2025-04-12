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
///   (or equivalently, the larger the `process` is), the less significant the overhead.
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
/// `skip_to_end` to consume the iterator. In the example setting, it is possible that other threads
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
///                         // will immediately consume the rest of the iterator
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
pub trait ConcurrentIter: Send + Sync {
    type Item: Send + Sync;

    type SequentialIter: Iterator<Item = Self::Item>;

    type ChunkPuller<'i>: ChunkPuller<ChunkItem = Self::Item>
    where
        Self: 'i;

    // transform

    fn into_seq_iter(self) -> Self::SequentialIter;

    // iterate

    fn skip_to_end(&self);

    fn next(&self) -> Option<Self::Item>;

    fn next_with_idx(&self) -> Option<(usize, Self::Item)>;

    // len

    fn size_hint(&self) -> (usize, Option<usize>);

    fn try_get_len(&self) -> Option<usize> {
        match self.size_hint() {
            (x, Some(y)) if x == y => Some(x),
            _ => None,
        }
    }

    // pullers

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_>;

    fn item_puller(&self) -> ItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    fn item_puller_with_idx(&self) -> EnumeratedItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    // provided transformations

    fn copied<'a, T>(self) -> ConIterCopied<'a, Self, T>
    where
        T: Send + Sync + Copy,
        Self: ConcurrentIter<Item = &'a T> + Sized,
    {
        ConIterCopied::new(self)
    }

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

    fn find<T, F>(
        num_threads: usize,
        con_iter: impl ConcurrentIter<Item = T>,
        predicate: F,
    ) -> Option<T>
    where
        T: Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
    {
        std::thread::scope(|s| {
            let mut results = vec![];
            for _ in 0..num_threads {
                results.push(s.spawn(|| {
                    for value in con_iter.item_puller() {
                        if predicate(&value) {
                            // will immediately consume the rest of the iterator
                            con_iter.skip_to_end();
                            return Some(value);
                        }
                    }
                    None
                }));
            }
            results.into_iter().filter_map(|x| x.join().unwrap()).next()
        })
    }

    let data: Vec<_> = (0..1000).map(|x| x.to_string()).collect();
    let value = find(4, data.con_iter(), |x| x.starts_with("33"));

    assert_eq!(value, Some(&33.to_string()));
}
