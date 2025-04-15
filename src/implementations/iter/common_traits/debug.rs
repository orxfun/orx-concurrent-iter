use crate::{ConcurrentIter, implementations::ConIterOfIter};
use core::fmt::Debug;

impl<I> Debug for ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConIterOfIter")
            .field("size_hint", &self.size_hint())
            .finish()
    }
}
