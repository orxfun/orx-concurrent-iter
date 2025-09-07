use crate::{IntoConcurrentIter, chain::ChainKnownLenI, concurrent_iter::ConcurrentIter};

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

    /// Creates a chain of this and `other` concurrent iterators.
    ///
    /// It is preferable to call `chain` over [`chain_inexact`] whenever the first iterator
    /// implements `ExactSizeConcurrentIter`.
    ///
    /// [`chain_inexact`]: crate::ConcurrentIter::chain_inexact
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let s1 = "abc".chars(); // exact iter
    /// let s2 = vec!['d', 'e', 'f'];
    ///
    /// let chain = s1.iter_into_con_iter().chain_inexact(s2);
    ///
    /// assert_eq!(chain.next(), Some('a'));
    /// assert_eq!(chain.next(), Some('b'));
    /// assert_eq!(chain.next(), Some('c'));
    /// assert_eq!(chain.next(), Some('d'));
    /// assert_eq!(chain.next(), Some('e'));
    /// assert_eq!(chain.next(), Some('f'));
    /// assert_eq!(chain.next(), None);
    /// ```
    fn chain<C>(self, other: C) -> ChainKnownLenI<Self, C::IntoIter>
    where
        C: IntoConcurrentIter<Item = Self::Item>,
        Self: Sized,
    {
        let len_i = self.len();
        ChainKnownLenI::new(self, other.into_con_iter(), len_i)
    }
}
