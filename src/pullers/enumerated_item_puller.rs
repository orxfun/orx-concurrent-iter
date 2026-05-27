use crate::concurrent_iter::ConcurrentIter;

/// A regular [`Iterator`] which is created from and linked to and
/// pulls its elements from a [`ConcurrentIter`].
///
/// It can be created using the [`item_puller_with_idx`] method of a concurrent iterator.
///
/// This is similar to [`ItemPuller`] except that this iterator additionally returns the
/// indices of the elements in the source concurrent iterator.
///
/// [`item_puller_with_idx`]: crate::ConcurrentIter::item_puller_with_idx
/// [`ItemPuller`]: crate::ItemPuller
///
/// # Examples
///
/// See the [`ItemPuller`] for detailed examples.
/// The following example only demonstrates the additional index that is returned by the
/// next method of the `EnumeratedItemPuller`.
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
///             for (idx, value) in con_iter.item_puller_with_idx() {
///                 assert_eq!(value, &idx.to_string());
///             }
///         });
///     }
/// });
/// ```
pub struct EnumeratedItemPuller<'a, I>
where
    I: ConcurrentIter,
{
    con_iter: &'a I,
}

impl<I: ConcurrentIter> EnumeratedItemPuller<'_, I> {
    /// Behaves exactly as `next` but additionally provides `thread_idx` to the iterator.
    /// This information might be useful for certain concurrent iterators, such as the
    /// [recursive concurrent iterator](https://crates.io/crates/orx-concurrent-recursive-iter).
    ///
    /// Assuming a program using `n` threads that accesses this iterator, `thread_idx` is
    /// assumed to be the internal ordering within this pool of threads taking values in
    /// `0..n`.
    #[inline(always)]
    pub fn next_by(&mut self, thread_idx: usize) -> Option<(usize, I::Item)> {
        self.con_iter.next_with_idx_by(thread_idx)
    }
}

impl<'i, I> From<&'i I> for EnumeratedItemPuller<'i, I>
where
    I: ConcurrentIter,
{
    fn from(con_iter: &'i I) -> Self {
        Self { con_iter }
    }
}

impl<I> Iterator for EnumeratedItemPuller<'_, I>
where
    I: ConcurrentIter,
{
    type Item = (usize, I::Item);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next_with_idx()
    }
}
