use crate::implementations::ConIterOfIter;

pub trait IterIntoConcurrentIter: Iterator + Sized
where
    Self::Item: Send + Sync,
{
    fn iter_into_con_iter(self) -> ConIterOfIter<Self>;
}

impl<I> IterIntoConcurrentIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_con_iter(self) -> ConIterOfIter<Self> {
        ConIterOfIter::new(self)
    }
}
