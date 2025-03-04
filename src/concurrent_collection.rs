use crate::{concurrent_iterable::ConcurrentIterable, into_concurrent_iter::IntoConcurrentIter};

pub trait ConcurrentCollection {
    type Item;

    type Iterable<'i>: ConcurrentIterable<Item = &'i Self::Item>
    where
        Self: 'i;

    fn as_concurrent_iterable(&self) -> Self::Iterable<'_>;

    fn con_iter(&self) -> <Self::Iterable<'_> as ConcurrentIterable>::Iter {
        self.as_concurrent_iterable().con_iter()
    }
}

impl<X> ConcurrentCollection for X
where
    X: IntoConcurrentIter,
    for<'a> &'a X: IntoConcurrentIter<Item = &'a <X as IntoConcurrentIter>::Item>,
{
    type Item = <X as IntoConcurrentIter>::Item;

    type Iterable<'i>
        = &'i X
    where
        Self: 'i;

    fn as_concurrent_iterable(&self) -> Self::Iterable<'_> {
        self
    }
}
