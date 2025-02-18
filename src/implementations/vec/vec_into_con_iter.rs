use super::con_iter_vec::ConIterVec;
use crate::{
    implementations::slice::con_iter_slice_ref::ConIterSliceRef,
    into_concurrent_iter::IntoConcurrentIter,
};
use alloc::vec::Vec;

impl<T> IntoConcurrentIter for Vec<T>
where
    T: Send + Sync,
{
    type Item = T;

    type Iter = ConIterVec<T>;

    fn into_concurrent_iter(self) -> Self::Iter {
        Self::Iter::new(self)
    }
}

impl<'a, T> IntoConcurrentIter for &'a Vec<T>
where
    T: Send + Sync,
{
    type Item = &'a T;

    type Iter = ConIterSliceRef<'a, T>;

    fn into_concurrent_iter(self) -> Self::Iter {
        Self::Iter::new(self.as_slice())
    }
}
