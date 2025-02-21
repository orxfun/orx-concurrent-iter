use super::con_iter_of_iter::ConIterOfIter;
use crate::ConcurrentIterable;
use orx_iterable::{transformations::CloningIterable, Iterable};

pub trait IterIntoConcurrentIter<T>: Iterator<Item = T> + Sized
where
    T: Send + Sync,
{
    fn iter_into_concurrent_iter(self) -> ConIterOfIter<Self, T>;
}

impl<T, I> IterIntoConcurrentIter<T> for I
where
    I: Iterator<Item = T>,
    T: Send + Sync,
{
    fn iter_into_concurrent_iter(self) -> ConIterOfIter<Self, T> {
        ConIterOfIter::new(self)
    }
}

impl<I> ConcurrentIterable for CloningIterable<I>
where
    I: Iterator + Clone,
    I::Item: Send + Sync,
{
    type Item = I::Item;

    type Iter = ConIterOfIter<I, I::Item>;

    fn concurrent_iter(&self) -> Self::Iter {
        ConIterOfIter::new(self.iter())
    }
}
