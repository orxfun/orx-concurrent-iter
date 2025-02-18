use crate::{concurrent_iter::ConcurrentIter, IntoConcurrentIter};

pub trait ConcurrentIterable {
    type Item: Send + Sync;

    type Iter: ConcurrentIter<Item = Self::Item>;

    fn concurrent_iter(&self) -> Self::Iter;
}

// impl

impl<'a, X> ConcurrentIterable for &'a X
where
    &'a X: IntoConcurrentIter,
{
    type Item = <&'a X as IntoConcurrentIter>::Item;

    type Iter = <&'a X as IntoConcurrentIter>::IntoIter;

    fn concurrent_iter(&self) -> Self::Iter {
        self.into_concurrent_iter()
    }
}

impl<'a, T> ConcurrentIterable for &'a [T]
where
    T: Send + Sync,
{
    type Item = <&'a [T] as IntoConcurrentIter>::Item;

    type Iter = <&'a [T] as IntoConcurrentIter>::IntoIter;

    fn concurrent_iter(&self) -> Self::Iter {
        self.into_concurrent_iter()
    }
}
