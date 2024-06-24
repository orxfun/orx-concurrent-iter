use crate::ConcurrentIter;

/// A type that can be consumed and turned into a concurrent iterator with `into_con_iter` method.
pub trait IntoConcurrentIter {
    /// Type of the items that the iterator yields.
    type Item;

    /// Concurrent iterator that this type will be converted into with the `into_con_iter` method.
    type ConIter: ConcurrentIter<Item = Self::Item>;

    /// Consumes this type and converts it into a concurrent iterator.
    fn into_con_iter(self) -> Self::ConIter;
}

/// An Iterator type that can be consumed and turned into a concurrent iterator with `into_con_iter` method.
pub trait IterIntoConcurrentIter {
    /// Type of the items that the iterator yields.
    type Item;

    /// Concurrent iterator that this type will be converted into with the `into_con_iter` method.
    type ConIter: ConcurrentIter<Item = Self::Item>;

    /// Consumes this type and converts it into a concurrent iterator.
    fn into_con_iter(self) -> Self::ConIter;
}
