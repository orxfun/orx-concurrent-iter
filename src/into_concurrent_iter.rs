use crate::concurrent_iter::ConcurrentIter;

pub trait IntoConcurrentIter {
    type Item: Send + Sync;

    type Iter: ConcurrentIter<Item = Self::Item>;

    fn into_concurrent_iter(self) -> Self::Iter;
}
