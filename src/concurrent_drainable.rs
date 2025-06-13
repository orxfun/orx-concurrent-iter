use crate::ConcurrentIter;
use core::ops::RangeBounds;

/// A type which can create a concurrent draining iterator over any of its sub-slices.
pub trait ConcurrentDrainableOverSlice {
    /// Type of draining iterator elements.
    type Item;

    /// Type of the draining iterator created by `con_drain` method.
    type DrainingIter<'a>: ConcurrentIter<Item = Self::Item>
    where
        Self: 'a;

    /// Creates a concurrent draining iterators such that:
    ///
    /// * the iterator yields all elements of the slice defined by the `range`,
    /// * this slice will be removed from the original collection (`self`).
    ///
    /// If the iterator is dropped before being fully consumed, it drops the remaining removed elements.
    ///
    /// If the complete range is provided (`..` or `0..self.len()`), self will remain empty.
    ///
    /// # Panics:
    ///
    /// * if the starting point of the `range` is greater than the ending point; or
    /// * if the ending point of the `range` is greater than `vec.len()`.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to mem::forget, for example),
    /// `self` may have lost and leaked elements arbitrarily, including elements outside the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_iter::*;
    ///
    /// let mut v = vec![1, 2, 3];
    /// let u: Vec<_> = v.con_drain(1..).item_puller().collect();
    ///
    /// assert_eq!(v, &[1]);
    /// assert_eq!(u, &[2, 3]);
    /// ```
    fn con_drain<R>(&mut self, range: R) -> Self::DrainingIter<'_>
    where
        R: RangeBounds<usize>;
}
