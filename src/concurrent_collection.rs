use crate::{concurrent_iterable::ConcurrentIterable, into_concurrent_iter::IntoConcurrentIter};

/// A type implementing [`ConcurrentCollection`] is a collection owning the elements such that
///
/// * if the elements are of type `T`,
/// * then, non-consuming [`con_iter`] method can be called **multiple times** to create concurrent
///   iterators; i.e., [`ConcurrentIter`], yielding references to the elements `&T`.
///
/// This trait can be considered as the *concurrent counterpart* of the [`Collection`] trait.
///
/// [`con_iter`]: crate::ConcurrentCollection::con_iter
/// [`Collection`]: orx_iterable::Collection
/// [`ConcurrentIter`]: crate::ConcurrentIter
///
/// # Examples
///
/// ```rust
/// use orx_concurrent_iter::*;
///
/// let data = vec![1, 2];
///
/// let con_iter = data.con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub trait ConcurrentCollection {
    /// Type of the element that the concurrent iterator yields.
    type Item;

    /// Type of the ConcurrentIterable that reference of this type implements.
    type Iterable<'i>: ConcurrentIterable<Item = &'i Self::Item>
    where
        Self: 'i;

    /// Returns the ConcurrentIterable that a reference of this type can create.
    fn as_concurrent_iterable(&self) -> Self::Iterable<'_>;

    /// A type implementing [`ConcurrentCollection`] is a collection owning the elements such that
    ///
    /// * if the elements are of type `T`,
    /// * then, non-consuming [`con_iter`] method can be called **multiple times** to create concurrent
    ///   iterators; i.e., [`ConcurrentIter`], yielding references to the elements `&T`.
    ///
    /// This trait can be considered as the *concurrent counterpart* of the [`Collection`] trait.
    ///
    /// [`con_iter`]: crate::ConcurrentCollection::con_iter
    /// [`Collection`]: orx_iterable::Collection
    /// [`ConcurrentIter`]: crate::ConcurrentIter
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let data = vec![1, 2];
    ///
    /// let con_iter = data.con_iter();
    /// assert_eq!(con_iter.next(), Some(&1));
    /// assert_eq!(con_iter.next(), Some(&2));
    /// assert_eq!(con_iter.next(), None);
    /// ```
    fn con_iter(&self) -> <Self::Iterable<'_> as ConcurrentIterable>::Iter {
        self.as_concurrent_iterable().con_iter()
    }
}

impl<X> ConcurrentCollection for X
where
    X: IntoConcurrentIter,
    for<'a> &'a X: IntoConcurrentIter<Item = &'a <X as IntoConcurrentIter>::Item>,
{
    type Item = <X as IntoConcurrentIter>::Item;

    type Iterable<'i>
        = &'i X
    where
        Self: 'i;

    fn as_concurrent_iterable(&self) -> Self::Iterable<'_> {
        self
    }
}
