use crate::implementations::ConIterOfIter;

/// Any regular iterator implements [`IterIntoConcurrentIter`] trait allowing them to be used
/// as a concurrent iterator; i.e., [`ConcurrentIter`], by calling [`iter_into_con_iter`].
///
/// Pulling of elements from the iterator are synchronized and safely shared to threads.
///
/// Therefore, converting an iterator into a concurrent iterator is most useful whenever
/// the work to be done on each element is a larger task than just yielding elements by the
/// underlying collection or generator.
///
/// [`iter_into_con_iter`]: crate::IterIntoConcurrentIter::iter_into_con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let data: Vec<_> = (0..123).collect();
/// let iter = data.iter().filter(|x| *x % 2 == 0).map(|x| x.to_string());
/// let con_iter = iter.iter_into_con_iter();
///
/// let num_threads = 4;
/// let sum_evens = std::thread::scope(|s| {
///     let mut handles = vec![];
///     for _ in 0..num_threads {
///         handles.push(s.spawn(|| {
///             let mut sum = 0;
///             for x in con_iter.item_puller() {
///                 let number: u64 = x.parse().unwrap();
///                 sum += number;
///             }
///             sum
///         }));
///     }
///     let mut final_sum = 0;
///     for h in handles {
///         final_sum += h.join().unwrap();
///     }
///     final_sum
/// });
///
/// assert_eq!(sum_evens, 3782);
/// ```
pub trait IterIntoConcurrentIter: Iterator + Sized
where
    Self::Item: Send + Sync,
{
    /// Any regular iterator implements [`IterIntoConcurrentIter`] trait allowing them to be used
    /// as a concurrent iterator; i.e., [`ConcurrentIter`], by calling [`iter_into_con_iter`].
    ///
    /// Pulling of elements from the iterator are synchronized and safely shared to threads.
    ///
    /// Therefore, converting an iterator into a concurrent iterator is most useful whenever
    /// the work to be done on each element is a larger task than just yielding elements by the
    /// underlying collection or generator.
    ///
    /// [`iter_into_con_iter`]: crate::IterIntoConcurrentIter::iter_into_con_iter
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let data: Vec<_> = (0..123).collect();
    /// let iter = data.iter().filter(|x| *x % 2 == 0).map(|x| x.to_string());
    /// let con_iter = iter.iter_into_con_iter();
    ///
    /// let num_threads = 4;
    /// let sum_evens = std::thread::scope(|s| {
    ///     let mut handles = vec![];
    ///     for _ in 0..num_threads {
    ///         handles.push(s.spawn(|| {
    ///             let mut sum = 0;
    ///             for x in con_iter.item_puller() {
    ///                 let number: u64 = x.parse().unwrap();
    ///                 sum += number;
    ///             }
    ///             sum
    ///         }));
    ///     }
    ///     let mut final_sum = 0;
    ///     for h in handles {
    ///         final_sum += h.join().unwrap();
    ///     }
    ///     final_sum
    /// });
    ///
    /// assert_eq!(sum_evens, 3782);
    /// ```
    fn iter_into_con_iter(self) -> ConIterOfIter<Self>;
}

impl<I> IterIntoConcurrentIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_con_iter(self) -> ConIterOfIter<Self> {
        ConIterOfIter::new(self)
    }
}
