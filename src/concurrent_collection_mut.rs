use crate::{ConcurrentCollection, ConcurrentIter, IntoConcurrentIter};

/// A type implementing [`ConcurrentCollectionMut`] is a collection owning the elements such that
///
/// * if the elements are of type `T`,
/// * then, non-consuming [`con_iter`] method can be called **multiple times** to create concurrent
///   iterators; i.e., [`ConcurrentIter`], yielding references to the elements `&T`; and further,
/// * non-consuming mutable [`con_iter_mut`] method can be called to create concurrent iterators
///   yielding mutable references to elements `&mut T`.
///
/// This trait can be considered as the *concurrent counterpart* of the [`CollectionMut`] trait.
///
/// [`con_iter`]: crate::ConcurrentCollection::con_iter
/// [`CollectionMut`]: orx_iterable::CollectionMut
/// [`ConcurrentIter`]: crate::ConcurrentIter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let mut data = vec![1, 2];
///
/// let con_iter = data.con_iter_mut();
/// assert_eq!(con_iter.next(), Some(&mut 1));
/// assert_eq!(con_iter.next(), Some(&mut 2));
/// assert_eq!(con_iter.next(), None);
///
/// let con_iter = data.con_iter_mut();
/// while let Some(x) = con_iter.next() {
///     *x *= 100;
/// }
/// assert_eq!(data, vec![100, 200]);
/// ```
pub trait ConcurrentCollectionMut: ConcurrentCollection {
    /// Type of the mutable concurrent iterator that this type can create with `con_iter_mut` method.
    type IterMut<'a>: ConcurrentIter<Item = &'a mut Self::Item>
    where
        Self: 'a;

    /// Creates a concurrent iterator to mutable references of the underlying data; i.e., `&mut Self::Item`.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let mut data = vec![1, 2];
    ///
    /// let con_iter = data.con_iter_mut();
    /// assert_eq!(con_iter.next(), Some(&mut 1));
    /// assert_eq!(con_iter.next(), Some(&mut 2));
    /// assert_eq!(con_iter.next(), None);
    ///
    /// let con_iter = data.con_iter_mut();
    /// while let Some(x) = con_iter.next() {
    ///     *x *= 100;
    /// }
    /// assert_eq!(data, vec![100, 200]);
    /// ```
    fn con_iter_mut(&mut self) -> Self::IterMut<'_>;
}

impl<X> ConcurrentCollectionMut for X
where
    X: IntoConcurrentIter,
    for<'a> &'a X: IntoConcurrentIter<Item = &'a <X as IntoConcurrentIter>::Item>,
    for<'a> &'a mut X: IntoConcurrentIter<Item = &'a mut <X as IntoConcurrentIter>::Item>,
{
    type IterMut<'a>
        = <&'a mut X as IntoConcurrentIter>::IntoIter
    where
        Self: 'a;

    fn con_iter_mut(&mut self) -> Self::IterMut<'_> {
        <&mut X as IntoConcurrentIter>::into_con_iter(self)
    }
}
