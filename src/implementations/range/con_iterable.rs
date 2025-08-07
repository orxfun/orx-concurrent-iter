use super::con_iter::ConIterRange;
use crate::concurrent_iterable::ConcurrentIterable;
use core::ops::Range;

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
