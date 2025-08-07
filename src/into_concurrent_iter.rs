use crate::concurrent_iter::ConcurrentIter;

/// Trait to convert a source (collection or generator) into a concurrent iterator; i.e., [`ConcurrentIter`],
/// using its [`into_con_iter`] method.
///
/// It can be considered as the *concurrent counterpart* of the [`IntoIterator`] trait.
///
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec![1, 2];
/// let con_iter = vec.into_con_iter();
/// assert_eq!(con_iter.next(), Some(1));
/// assert_eq!(con_iter.next(), Some(2));
/// assert_eq!(con_iter.next(), None);
///
/// let range = 11..13;
/// let con_iter = range.into_con_iter();
/// assert_eq!(con_iter.next(), Some(11));
/// assert_eq!(con_iter.next(), Some(12));
/// assert_eq!(con_iter.next(), None);
/// ```
pub trait IntoConcurrentIter {
    /// Type of the element that the concurrent iterator yields.
    type Item;

    /// Type of the concurrent iterator that this type can be converted into.
    type IntoIter: ConcurrentIter<Item = Self::Item>;

    /// Trait to convert a source (collection or generator) into a concurrent iterator; i.e., [`ConcurrentIter`],
    /// using its [`into_con_iter`] method.
    ///
    /// It can be considered as the *concurrent counterpart* of the [`IntoIterator`] trait.
    ///
    /// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec![1, 2];
    /// let con_iter = vec.into_con_iter();
    /// assert_eq!(con_iter.next(), Some(1));
    /// assert_eq!(con_iter.next(), Some(2));
    /// assert_eq!(con_iter.next(), None);
    ///
    /// let range = 11..13;
    /// let con_iter = range.into_con_iter();
    /// assert_eq!(con_iter.next(), Some(11));
    /// assert_eq!(con_iter.next(), Some(12));
    /// assert_eq!(con_iter.next(), None);
    /// ```
    fn into_con_iter(self) -> Self::IntoIter;
}
