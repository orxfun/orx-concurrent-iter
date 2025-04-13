use super::ChunkPuller;
use core::iter::Enumerate;

/// Flattened version of a [`ChunkPuller`] which conveniently implements [`Iterator`].
///
/// Similar to the regular chunk puller, a flattened enumerated chunk puller is created from and
/// linked to and pulls its elements from a [`ConcurrentIter`].
///
/// It can be created by calling the [`flattened_with_idx`] method on a chunk puller that is
/// created by the [`chunk_puller`] method of a concurrent iterator.
///
/// Unlike the [`FlattenedChunkPuller`], flattened enumerated chunk puller additionally returns
/// the indices of elements.
///
/// [`ChunkPuller`]: crate::ChunkPuller
/// [`ConcurrentIter`]: crate::ConcurrentIter
/// [`chunk_puller`]: crate::ConcurrentIter::chunk_puller
/// [`flattened_with_idx`]: crate::ChunkPuller::flattened_with_idx
/// [`FlattenedChunkPuller`]: crate::FlattenedChunkPuller
///
/// # Examples
///
/// See the [`FlattenedChunkPuller`] for detailed examples.
/// The following example only demonstrates the additional index that is returned by the
/// next method of the `FlattenedEnumeratedChunkPuller`.
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
///             for (idx, value) in con_iter.chunk_puller(8).flattened_with_idx() {
///                 assert_eq!(value, &idx.to_string());
///             }
///         });
///     }
/// });
/// ```
pub struct FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    puller: P,
    current_begin_idx: usize,
    current_chunk: Enumerate<P::Chunk<'c>>,
}

impl<'c, P> From<P> for FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    fn from(puller: P) -> Self {
        Self {
            puller,
            current_begin_idx: 0,
            current_chunk: Default::default(),
        }
    }
}

impl<'c, P> FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    /// Converts the flattened chunk puller back to the chunk puller it
    /// is created from.
    pub fn into_chunk_puller(self) -> P {
        self.puller
    }

    fn next_chunk(&mut self) -> Option<(usize, P::ChunkItem)> {
        let puller = unsafe { &mut *(&mut self.puller as *mut P) };
        match puller.pull_with_idx() {
            Some((begin_idx, chunk)) => {
                self.current_begin_idx = begin_idx;
                self.current_chunk = chunk.enumerate();
                self.next()
            }
            None => None,
        }
    }
}

impl<'c, P> Iterator for FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    type Item = (usize, P::ChunkItem);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_chunk.next() {
            Some((i, x)) => Some((self.current_begin_idx + i, x)),
            None => self.next_chunk(),
        }
    }
}
