use super::con_iter_of_iter::ConIterOfIter;
use crate::ConcurrentIterable;
use orx_iterable::{transformations::CloningIterable, Iterable};

pub trait IterIntoConcurrentIter: Iterator + Sized
where
    Self::Item: Send + Sync,
{
    fn iter_into_concurrent_iter(self) -> ConIterOfIter<Self>;
}

impl<I> IterIntoConcurrentIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_concurrent_iter(self) -> ConIterOfIter<Self> {
        ConIterOfIter::new(self)
    }
}

impl<I> ConcurrentIterable for CloningIterable<I>
where
    I: Iterator + Clone,
    I::Item: Send + Sync,
{
    type Item = I::Item;

    type Iter = ConIterOfIter<I>;

    fn concurrent_iter(&self) -> Self::Iter {
        ConIterOfIter::new(self.iter())
    }
}
