use super::con_iter::ConIterRange;
use crate::{
    concurrent_iterable::ConcurrentIterable,
    implementations::range::into_con_iter::range_inclusive_to_range,
};
use core::ops::{Range, RangeInclusive};

impl<T> ConcurrentIterable for Range<T>
where
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type Iter = ConIterRange<T>;

    fn con_iter(&self) -> Self::Iter {
        Self::Iter::new(self.clone())
    }
}

impl<T> ConcurrentIterable for RangeInclusive<T>
where
    T: Send + From<usize> + Into<usize> + Clone,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type Iter = ConIterRange<T>;

    fn con_iter(&self) -> Self::Iter {
        range_inclusive_to_range(self.clone()).con_iter()
    }
}
