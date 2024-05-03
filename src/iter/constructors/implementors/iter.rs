use crate::{iter::constructors::into_con_iter::IterIntoConcurrentIter, ConIterOfIter};

impl<T: Send + Sync, Iter> IterIntoConcurrentIter for Iter
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    type ConIter = ConIterOfIter<T, Iter>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }

    fn try_get_exact_len(&self) -> Option<usize> {
        None
    }
}
