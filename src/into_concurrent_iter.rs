use crate::concurrent_iter::ConcurrentIter;

pub trait IntoConcurrentIter {
    type Item: Send + Sync;

    type IntoIter: ConcurrentIter<Item = Self::Item>;

    fn into_con_iter(self) -> Self::IntoIter;
}
