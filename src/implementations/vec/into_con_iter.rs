use super::con_iter::ConIterVec;
use crate::{
    implementations::{slice::ConIterSlice, slice_mut::ConIterSliceMut},
    into_concurrent_iter::IntoConcurrentIter,
};
use alloc::vec::Vec;

impl<T> IntoConcurrentIter for Vec<T> {
    type Item = T;

    type IntoIter = ConIterVec<T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<'a, T> IntoConcurrentIter for &'a Vec<T>
where
    T: Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterSlice<'a, T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self.as_slice())
    }
}

impl<'a, T> IntoConcurrentIter for &'a mut Vec<T>
where
    T: Send,
{
    type Item = &'a mut T;

    type IntoIter = ConIterSliceMut<'a, T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self.as_mut_slice())
    }
}
