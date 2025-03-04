use crate::{concurrent_iter::ConcurrentIter, into_concurrent_iter::IntoConcurrentIter};

pub trait ConcurrentIterable {
    type Item: Send + Sync;

    type Iter: ConcurrentIter<Item = Self::Item>;

    fn con_iter(&self) -> Self::Iter;
}

// impl

impl<'a, X> ConcurrentIterable for &'a X
where
    &'a X: IntoConcurrentIter,
{
    type Item = <&'a X as IntoConcurrentIter>::Item;

    type Iter = <&'a X as IntoConcurrentIter>::IntoIter;

    fn con_iter(&self) -> Self::Iter {
        self.into_con_iter()
    }
}
