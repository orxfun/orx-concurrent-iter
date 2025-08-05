use crate::concurrent_iter::ConcurrentIter;

/// A regular [`Iterator`] which is created from and linked to and
/// pulls its elements from a [`ConcurrentIter`].
///
/// It can be created using the [`item_puller`] method of a concurrent iterator.
///
/// [`item_puller`]: crate::ConcurrentIter::item_puller
///
/// # Examples
///
/// The definition might sound a bit confusing.
///
/// The following example demonstrates how it works:
///
/// * We have a concurrent iterator `con_iter` over elements "0", "1", ..., "99".
/// * We spawn two threads, say A and B.
/// * Each thread creates an `ItemPuller`, named as `puller`, from the same `con_iter`.
/// * The following is one possible sequence of this parallel execution.
///   * Thread A pulls "0" and calls process("0").
///   * Thread B pulls "1" and calls process("1"); it completes processing before A.
///   * Thread B pulls "2" and calls process("2").
///   * Thread A pulls "3" and calls process("3").
///   * and so on until all 100 elements are processed by the two threads.
/// * Notice that there is only one data source `con_iter`;
///   both of the `puller`s are connected to the same concurrent iterator, and
///   each element is processed only once.
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let num_threads = 2;
/// let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
/// let con_iter = data.con_iter();
///
/// let process = |_x: &String| { /* assume actual work */ };
///
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             // puller implements Iterator
///             let puller = con_iter.item_puller();
///             for value in puller {
///                 process(value);
///             }
///         });
///     }
/// });
/// ```
///
/// This approach brings the convenience of regular Iterators to the concurrent code.
/// The example above already demonstrates that we can now use a regular `for` loop as we are
/// writing a sequential code, while the program is parallelized.
///
/// Actually, we could've written an equivalent of the above program by directly using the
/// concurrent iterator's [`next`] method and a `while let` loop:
///
/// ```ignore
/// // ...
/// std::thread::scope(|s| {
///     for _ in 0..num_threads {
///         s.spawn(|| {
///             while let Some(value) = con_iter.next() {
///                 process(value);
///             }
///         });
///     }
/// });
/// ```
///
/// [`next`]: crate::ConcurrentIter::next
///
/// However, the convenience of the pullers goes beyond the `for` loops.
/// All beautiful ergonomic Iterator methods become available in concurrent programs.
///
/// The following example demonstrate a very simple yet efficient implementation of the
/// parallelized version of the [`reduce`] operation.
///
/// Notice that the entire implementation is nothing but a chain of Iterator methods.
///
/// [`reduce`]: Iterator::reduce
///
/// ```
/// use orx_concurrent_iter::*;
///
/// fn parallel_reduce<T, F>(
///     num_threads: usize,
///     con_iter: impl ConcurrentIter<Item = T> + Sync,
///     reduce: F,
/// ) -> Option<T>
/// where
///     T: Send,
///     F: Fn(T, T) -> T + Sync,
/// {
///     std::thread::scope(|s| {
///         (0..num_threads)
///             .map(|_| s.spawn(|| con_iter.item_puller().reduce(&reduce))) // reduce inside each thread
///             .filter_map(|x| x.join().unwrap()) // join threads
///             .reduce(&reduce) // reduce thread results to final result
///     })
/// }
///
/// let sum = parallel_reduce(8, (0..0).into_con_iter(), |a, b| a + b);
/// assert_eq!(sum, None);
///
/// let n = 10_000;
/// let data: Vec<_> = (0..n).collect();
/// let sum = parallel_reduce(8, data.con_iter().copied(), |a, b| a + b);
/// assert_eq!(sum, Some(n * (n - 1) / 2));
/// ```
pub struct ItemPuller<'a, I>
where
    I: ConcurrentIter,
{
    con_iter: &'a I,
}

impl<'i, I> From<&'i I> for ItemPuller<'i, I>
where
    I: ConcurrentIter,
{
    fn from(con_iter: &'i I) -> Self {
        Self { con_iter }
    }
}

impl<I> Iterator for ItemPuller<'_, I>
where
    I: ConcurrentIter,
{
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next()
    }
}
