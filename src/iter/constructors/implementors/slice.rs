use crate::{
    iter::constructors::con_iterable::ConcurrentIterable, ConIterOfSlice, IntoConcurrentIter,
};

impl<T: Send + Sync> ConcurrentIterable for &[T] {
    type Item<'i>
        = &'i T
    where
        Self: 'i;

    type ConIter<'i>
        = ConIterOfSlice<'i, T>
    where
        Self: 'i;

    fn con_iter(&self) -> Self::ConIter<'_> {
        Self::ConIter::new(self)
    }
}

impl<'a, T: Send + Sync> IntoConcurrentIter for &'a [T] {
    type Item = &'a T;

    type ConIter = ConIterOfSlice<'a, T>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}
