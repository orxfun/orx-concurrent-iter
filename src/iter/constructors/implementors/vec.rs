use crate::{
    iter::implementors::vec::ConIterOfVec, ConIterOfSlice, ConcurrentIterable, IntoConcurrentIter,
};

impl<T: Send + Sync + Default> ConcurrentIterable for Vec<T> {
    type Item<'i> = &'i T where Self: 'i;

    type ConIter<'i> = ConIterOfSlice<'i, T> where Self: 'i;

    fn con_iter(&self) -> Self::ConIter<'_> {
        Self::ConIter::new(self.as_slice())
    }
}

impl<T: Send + Sync + Default> IntoConcurrentIter for Vec<T> {
    type Item = T;

    type ConIter = ConIterOfVec<T>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}
