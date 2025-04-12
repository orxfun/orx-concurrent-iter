use crate::concurrent_iter::ConcurrentIter;

/// A concurrent iterator which has a certain information of the number of
/// remaining elements.
///
/// It can be considered as the *concurrent counterpart* of the [`ExactSizeIterator`].
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec!['x', 'y'];
///
/// let con_iter = vec.con_iter();
/// assert_eq!(con_iter.len(), 2);
///
/// assert_eq!(con_iter.next(), Some(&'x'));
/// assert_eq!(con_iter.len(), 1);
///
/// assert_eq!(con_iter.next(), Some(&'y'));
/// assert_eq!(con_iter.len(), 0);
/// assert!(con_iter.is_empty());
///
/// assert_eq!(con_iter.next(), None);
/// assert_eq!(con_iter.len(), 0);
/// assert!(con_iter.is_empty());
/// ```
pub trait ExactSizeConcurrentIter: ConcurrentIter {
    /// Returns the number remaining elements in the concurrent iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec!['x', 'y'];
    ///
    /// let con_iter = vec.con_iter();
    /// assert_eq!(con_iter.len(), 2);
    ///
    /// assert_eq!(con_iter.next(), Some(&'x'));
    /// assert_eq!(con_iter.len(), 1);
    ///
    /// assert_eq!(con_iter.next(), Some(&'y'));
    /// assert_eq!(con_iter.len(), 0);
    /// assert!(con_iter.is_empty());
    ///
    /// assert_eq!(con_iter.next(), None);
    /// assert_eq!(con_iter.len(), 0);
    /// assert!(con_iter.is_empty());
    /// ```
    fn len(&self) -> usize;

    /// Returns true if there are no elements left in the concurrent iterator.
    /// Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let vec = vec!['x', 'y'];
    ///
    /// let con_iter = vec.con_iter();
    /// assert!(!con_iter.is_empty());
    ///
    /// assert_eq!(con_iter.next(), Some(&'x'));
    /// assert!(!con_iter.is_empty());
    ///
    /// assert_eq!(con_iter.next(), Some(&'y'));
    /// assert!(con_iter.is_empty());
    ///
    /// assert_eq!(con_iter.next(), None);
    /// assert!(con_iter.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
