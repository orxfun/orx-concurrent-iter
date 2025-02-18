use super::con_iter_vec::ConIterVec;
use crate::into_concurrent_iter::IntoConcurrentIter;
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
