use crate::{concurrent_iter::ConcurrentIter, into_concurrent_iter::IntoConcurrentIter};

/// [`ConcurrentIterable`] types are those from which concurrent iterators can be created
/// **multiple times** using the [`con_iter`] method, since this method call does not consume the source.
///
/// This trait can be considered as the *concurrent counterpart* of the [`Iterable`] trait.
///
/// [`con_iter`]: crate::ConcurrentIterable::con_iter
/// [`Iterable`]: orx_iterable::Iterable
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec![1, 2]; // Vec<T>: ConcurrentIterable<Item = &T>
/// for _ in 0..5 {
///     let con_iter = vec.con_iter();
///     assert_eq!(con_iter.next(), Some(&1));
///     assert_eq!(con_iter.next(), Some(&2));
///     assert_eq!(con_iter.next(), None);
/// }
///
/// let slice = vec.as_slice(); // &[T]: ConcurrentIterable<Item = &T>
/// for _ in 0..5 {
///     let con_iter = slice.con_iter();
///     assert_eq!(con_iter.next(), Some(&1));
///     assert_eq!(con_iter.next(), Some(&2));
///     assert_eq!(con_iter.next(), None);
/// }
///
/// let range = 11..13; // Range<T>: ConcurrentIterable<Item = T>
/// for _ in 0..5 {
///     let con_iter = range.con_iter();
///     assert_eq!(con_iter.next(), Some(11));
///     assert_eq!(con_iter.next(), Some(12));
///     assert_eq!(con_iter.next(), None);
/// }
/// ```
pub trait ConcurrentIterable {
    /// Type of the element that the concurrent iterator yields.
    type Item;

    /// Type of the concurrent iterator that this type can create with `con_iter` method.
    type Iter: ConcurrentIter<Item = Self::Item>;

    /// [`ConcurrentIterable`] types are those from which concurrent iterators can be created
    /// **multiple times** using the [`con_iter`] method, since this method call does not consume the source.
    ///
    /// This trait can be considered as the *concurrent counterpart* of the [`Iterable`] trait.
    ///
    /// [`con_iter`]: crate::ConcurrentIterable::con_iter
    /// [`Iterable`]: orx_iterable::Iterable
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec![1, 2]; // Vec<T>: ConcurrentIterable<Item = &T>
    /// for _ in 0..5 {
    ///     let con_iter = vec.con_iter();
    ///     assert_eq!(con_iter.next(), Some(&1));
    ///     assert_eq!(con_iter.next(), Some(&2));
    ///     assert_eq!(con_iter.next(), None);
    /// }
    ///
    /// let slice = vec.as_slice(); // &[T]: ConcurrentIterable<Item = &T>
    /// for _ in 0..5 {
    ///     let con_iter = slice.con_iter();
    ///     assert_eq!(con_iter.next(), Some(&1));
    ///     assert_eq!(con_iter.next(), Some(&2));
    ///     assert_eq!(con_iter.next(), None);
    /// }
    ///
    /// let range = 11..13; // Range<T>: ConcurrentIterable<Item = T>
    /// for _ in 0..5 {
    ///     let con_iter = range.con_iter();
    ///     assert_eq!(con_iter.next(), Some(11));
    ///     assert_eq!(con_iter.next(), Some(12));
    ///     assert_eq!(con_iter.next(), None);
    /// }
    /// ```
    fn con_iter(&self) -> Self::Iter;
}

// impl

impl<'a, X> ConcurrentIterable for &'a X
where
    &'a X: IntoConcurrentIter,
{
    type Item = <&'a X as IntoConcurrentIter>::Item;

    type Iter = <&'a X as IntoConcurrentIter>::IntoIter;

    fn con_iter(&self) -> Self::Iter {
        self.into_con_iter()
    }
}
