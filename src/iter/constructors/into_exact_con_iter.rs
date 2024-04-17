use crate::ExactSizeConcurrentIter;

/// A type that can be consumed and turned into an exact size concurrent iterator with `into_exact_con_iter` method.
pub trait IntoExactSizeConcurrentIter {
    /// Type of the items that the iterator yields.
    type Item;

    /// Concurrent iterator that this type will be converted into with the `into_exact_con_iter` method.
    type ExactConIter: ExactSizeConcurrentIter<Item = Self::Item>;

    /// Consumes this type and converts it into an exact size concurrent iterator.
    fn into_exact_con_iter(self) -> Self::ExactConIter;

    /// Returns the exact remaining length of the exact size concurrent iterator, before converting it into the iterator.
    fn exact_len(&self) -> usize;
}
