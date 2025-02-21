use super::con_iter_slice::ConIterSlice;
use crate::{into_concurrent_iter::IntoConcurrentIter, ConcurrentIterable};

impl<'a, T> IntoConcurrentIter for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterSlice<'a, T>;

    fn into_concurrent_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<'a, T> ConcurrentIterable for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type Iter = ConIterSlice<'a, T>;

    fn concurrent_iter(&self) -> Self::Iter {
        Self::Iter::new(self)
    }
}
