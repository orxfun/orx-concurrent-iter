use super::con_iter_ref::ConIterVecDequeRef;
use crate::IntoConcurrentIter;
use std::collections::VecDeque;

impl<'a, T> IntoConcurrentIter for &'a VecDeque<T>
where
    T: Send + Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterVecDequeRef<'a, T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}
