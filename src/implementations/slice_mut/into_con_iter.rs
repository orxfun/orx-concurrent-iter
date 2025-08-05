use super::con_iter::ConIterSliceMut;
use crate::into_concurrent_iter::IntoConcurrentIter;

impl<'a, T: 'a> IntoConcurrentIter for &'a mut [T] {
    type Item = &'a mut T;

    type IntoIter = ConIterSliceMut<'a, T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}
