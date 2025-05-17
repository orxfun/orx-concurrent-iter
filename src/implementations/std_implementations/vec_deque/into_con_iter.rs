use super::con_iter_ref::ConIterVecDequeRef;
use crate::{IntoConcurrentIter, implementations::ConIterVec};
use alloc::{collections::VecDeque, vec::Vec};

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

impl<T> IntoConcurrentIter for VecDeque<T>
where
    T: Send + Sync,
{
    type Item = T;

    type IntoIter = ConIterVec<T>;

    fn into_con_iter(self) -> Self::IntoIter {
        Vec::from(self).into_con_iter()
    }
}
