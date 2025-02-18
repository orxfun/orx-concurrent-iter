use super::con_iter_slice_ref::ConIterSliceRef;
use crate::into_concurrent_iter::IntoConcurrentIter;

impl<'a, T> IntoConcurrentIter for &'a [T]
where
    T: Send + Sync,
{
    type Item = &'a T;

    type Iter = ConIterSliceRef<'a, T>;

    fn into_concurrent_iter(self) -> Self::Iter {
        Self::Iter::new(self)
    }
}
