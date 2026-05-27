use super::ChunkPuller;

/// Flattened version of a [`ChunkPuller`] which conveniently implements [`Iterator`].
///
/// Similar to the regular chunk puller, a flattened chunk puller is created from and
/// linked to and pulls its elements from a [`ConcurrentIter`].
///
/// It can be created by calling the [`flattened`] method on a chunk puller that is
/// created by the [`chunk_puller`] method of a concurrent iterator.
///
/// [`ChunkPuller`]: crate::ChunkPuller
/// [`ConcurrentIter`]: crate::ConcurrentIter
/// [`chunk_puller`]: crate::ConcurrentIter::chunk_puller
/// [`flattened`]: crate::ChunkPuller::flattened
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
///             .filter_map(|x| x.join().unwrap()) // join threads
///             .reduce(&reduce) // reduce thread results to final result
///     })
/// }
///
/// let sum = parallel_reduce(8, 64, (0..0).into_con_iter(), |a, b| a + b);
/// assert_eq!(sum, None);
///
/// let n = 10_000;
/// let data: Vec<_> = (0..n).collect();
/// let sum = parallel_reduce(8, 64, data.con_iter().copied(), |a, b| a + b);
/// assert_eq!(sum, Some(n * (n - 1) / 2));
/// ```
///
/// [`reduce`]: Iterator::reduce
/// [`ItemPuller`]: crate::ItemPuller
pub struct FlattenedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    puller: P,
    current_chunk: Option<P::Chunk<'c>>,
}

impl<P> From<P> for FlattenedChunkPuller<'_, P>
where
    P: ChunkPuller,
{
    fn from(puller: P) -> Self {
        Self {
            puller,
            current_chunk: None,
        }
    }
}

impl<P> FlattenedChunkPuller<'_, P>
where
    P: ChunkPuller,
{
    /// Converts the flattened chunk puller back to the chunk puller it
    /// is created from.
    pub fn into_chunk_puller(self) -> P {
        self.puller
    }

    fn next_chunk(&mut self) -> Option<P::ChunkItem> {
        let puller = unsafe { &mut *(&mut self.puller as *mut P) };
        match puller.pull() {
            Some(chunk) => {
                self.current_chunk = Some(chunk);
                self.next()
            }
            None => None,
        }
    }
}

impl<P> Iterator for FlattenedChunkPuller<'_, P>
where
    P: ChunkPuller,
{
    type Item = P::ChunkItem;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current_chunk {
            Some(chunk) => {
                let next = chunk.next();
                match next.is_some() {
                    true => next,
                    false => self.next_chunk(),
                }
            }
            None => self.next_chunk(),
        }
    }
}
