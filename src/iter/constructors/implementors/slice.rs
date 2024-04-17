use crate::{
    iter::constructors::{
        con_iterable::ConcurrentIterable, into_exact_con_iter::IntoExactSizeConcurrentIter,
    },
    ConIterOfSlice, IntoConcurrentIter,
};

impl<'a, T: Send + Sync> ConcurrentIterable for &'a [T] {
    type Item<'i> = &'i T where
    Self: 'i;

    type ConIter<'i> = ConIterOfSlice<'i, T> where
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

impl<'a, T: Send + Sync> IntoExactSizeConcurrentIter for &'a [T] {
    type Item = &'a T;

    type ExactConIter = ConIterOfSlice<'a, T>;

    fn into_exact_con_iter(self) -> Self::ExactConIter {
        Self::ExactConIter::new(self)
    }

    fn exact_len(&self) -> usize {
        self.len()
    }
}
