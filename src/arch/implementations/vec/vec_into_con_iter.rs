use super::con_iter_vec::ConIterVec;
use crate::{implementations::slice::ConIterSlice, into_concurrent_iter::IntoConcurrentIter};
use alloc::vec::Vec;

impl<T> IntoConcurrentIter for Vec<T>
where
    T: Send + Sync,
{
    type Item = T;

    type IntoIter = ConIterVec<T>;

    fn into_concurrent_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<'a, T> IntoConcurrentIter for &'a Vec<T>
where
    T: Send + Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterSlice<'a, T>;

    fn into_concurrent_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self.as_slice())
    }
}
