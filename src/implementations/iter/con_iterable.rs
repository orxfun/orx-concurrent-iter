use super::ConIterOfIter;
use crate::concurrent_iterable::ConcurrentIterable;
use orx_iterable::{transformations::CloningIterable, Iterable};

impl<I> ConcurrentIterable for CloningIterable<I>
where
    I: Iterator + Clone,
    I::Item: Send + Sync,
{
    type Item = I::Item;

    type Iter = ConIterOfIter<I>;

    fn con_iter(&self) -> Self::Iter {
        ConIterOfIter::new(self.iter())
    }
}
