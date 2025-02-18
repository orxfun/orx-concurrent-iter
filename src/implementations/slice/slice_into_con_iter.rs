use super::con_iter_slice_ref::ConIterSliceRef;
use crate::into_concurrent_iter::IntoConcurrentIter;

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
