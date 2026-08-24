use super::con_iter::ConIterRange;
use crate::into_concurrent_iter::IntoConcurrentIter;
use core::ops::{Range, RangeInclusive};

impl<T> IntoConcurrentIter for Range<T>
where
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type IntoIter = ConIterRange<T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<T> IntoConcurrentIter for RangeInclusive<T>
where
    T: Send + From<usize> + Into<usize> + Clone,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type IntoIter = ConIterRange<T>;

    fn into_con_iter(self) -> Self::IntoIter {
        range_inclusive_to_range(self).into_con_iter()
    }
}

pub(super) fn range_inclusive_to_range<T>(range_inclusive: RangeInclusive<T>) -> Range<T>
where
    T: Send + From<usize> + Into<usize> + Clone,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    let a = range_inclusive.start().clone();
    let b: usize = (range_inclusive.end().clone()).into().saturating_add(1);
    let b: T = b.into();
    a..b
}
