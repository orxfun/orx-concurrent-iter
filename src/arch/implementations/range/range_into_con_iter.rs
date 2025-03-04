use super::con_iter_range::ConIterRange;
use crate::{into_concurrent_iter::IntoConcurrentIter, ConcurrentIterable};
use core::ops::{Add, Range};

impl<T> IntoConcurrentIter for Range<T>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type IntoIter = ConIterRange<T>;

    fn into_concurrent_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<T> ConcurrentIterable for Range<T>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    Range<T>: Default + ExactSizeIterator<Item = T>,
{
    type Item = T;

    type Iter = ConIterRange<T>;

    fn concurrent_iter(&self) -> Self::Iter {
        Self::Iter::new(self.clone())
    }
}
