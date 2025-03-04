use super::con_iter::ConIterRange;
use crate::into_concurrent_iter::IntoConcurrentIter;
use core::ops::Range;

impl<T> IntoConcurrentIter for Range<T>
where
    T: Send + Sync + From<usize> + Into<usize>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type IntoIter = ConIterRange<T>;

    fn into_concurrent_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}
