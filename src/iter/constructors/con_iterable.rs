use crate::ConcurrentIter;

/// A type that is concurrently iterable; i.e., which can provide a `ConcurrentIter` with the `con_iter` method.
pub trait ConcurrentIterable {
    /// Type of the items that the iterator yields.
    type Item<'i>
    where
        Self: 'i;

    /// Concurrent iterator that this type creates with the `con_iter` method.
    type ConIter<'i>: ConcurrentIter<Item = Self::Item<'i>>
    where
        Self: 'i;

    /// Creates a concurrent iterator.
    fn con_iter(&self) -> Self::ConIter<'_>;
}
