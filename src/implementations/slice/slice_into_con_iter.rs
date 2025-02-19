use super::con_iter_slice::ConIterSliceRef;
use crate::{into_concurrent_iter::IntoConcurrentIter, ConcurrentIterable};

impl<'a, T> IntoConcurrentIter for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterSliceRef<'a, T>;

    fn into_concurrent_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<'a, T> ConcurrentIterable for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type Iter = ConIterSliceRef<'a, T>;

    fn concurrent_iter(&self) -> Self::Iter {
        Self::Iter::new(self)
    }
}
