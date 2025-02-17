use crate::iter::con_iter_x::ConcurrentIterX;

/// A type that can be consumed and turned into a concurrent iterator with `into_con_iter_x` method.
/// Note that the 'x' stands for unordered in a multi-threaded execution.
/// Note that:
/// * if we are iterating on a single thread, the elements will be iterated in the same order;
/// * however, if we are iterating by multiple threads, a `IntoConcurrentIterX` implementation cannot tell the original
///   index of a yielded element; unlike the default `IntoConcurrentIter` types.
///
/// If a type does not have the `into_con_iter_x` method (does not implement `IntoConcurrentIterX`), this means that:
/// * there is no advantage of losing track of order in terms of computation time;
/// * hence, these types only have the `con_iter` or `into_con_iter` methods.
///
/// If a type both has the `into_con_iter_x` and `into_con_iter` methods:
/// * We need to use the `into_con_iter` whenever we need to know the indices of the elements we receive from the iterator.
///   For instance, if we want to map elements and collect them in the same order of the inputs, correct indices would be a requirement.
/// * Otherwise, we can use `into_con_iter_x` which will most likely provide a performance improvement (it would not be implemented otherwise).
///   For instance, if we want to sum the elements in a collection, we can simply operate with an arbitrary order.
pub trait IntoConcurrentIterX {
    /// Type of the items that the iterator yields.
    type Item;

    /// Concurrent iterator that this type will be converted into with the `into_con_iter_x` method.
    type ConIter: ConcurrentIterX<Item = Self::Item>;

    /// Consumes this type and converts it into a concurrent iterator.
    fn into_con_iter_x(self) -> Self::ConIter;
}
