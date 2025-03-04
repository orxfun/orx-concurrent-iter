use super::con_iter::ConIterSlice;
use crate::{concurrent_iterable::ConcurrentIterable, into_concurrent_iter::IntoConcurrentIter};

impl<'a, T> IntoConcurrentIter for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterSlice<'a, T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<'a, T> ConcurrentIterable for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type Iter = ConIterSlice<'a, T>;

    fn con_iter(&self) -> Self::Iter {
        Self::Iter::new(self)
    }
}
