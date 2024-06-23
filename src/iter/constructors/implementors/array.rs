use crate::{
    iter::{
        constructors::{
            into_con_iter::IntoConcurrentIter, into_exact_con_iter::IntoExactSizeConcurrentIter,
        },
        implementors::array::ConIterOfArray,
    },
    ConIterOfSlice, ConcurrentIterable,
};

impl<const N: usize, T: Send + Sync + Default> ConcurrentIterable for [T; N] {
    type Item<'i> = &'i T where Self: 'i;

    type ConIter<'i> = ConIterOfSlice<'i, T> where Self: 'i;

    fn con_iter(&self) -> Self::ConIter<'_> {
        Self::ConIter::new(self.as_slice())
    }
}

impl<const N: usize, T: Send + Sync + Default> IntoConcurrentIter for [T; N] {
    type Item = T;

    type ConIter = ConIterOfArray<N, T>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}

impl<const N: usize, T: Send + Sync + Default> IntoExactSizeConcurrentIter for [T; N] {
    type Item = T;

    type ConIter = ConIterOfArray<N, T>;

    fn into_exact_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }

    fn exact_len(&self) -> usize {
        self.len()
    }
}
