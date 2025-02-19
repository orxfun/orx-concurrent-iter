use super::con_iter_copied::ConIterCopied;
use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Enumeration, Regular},
};

pub trait IntoCopiedConcurrentIter<'a, T, E = Regular>
where
    E: Enumeration,
    T: Send + Sync + Copy + 'a,
    Self: ConcurrentIter<E, Item = &'a T>,
{
    fn copied(self) -> ConIterCopied<'a, Self, T, E>;
}

impl<'a, T, E, I> IntoCopiedConcurrentIter<'a, T, E> for I
where
    E: Enumeration,
    T: Send + Sync + Copy + 'a,
    I: ConcurrentIter<E, Item = &'a T>,
{
    fn copied(self) -> ConIterCopied<'a, Self, T, E> {
        ConIterCopied::new(self)
    }
}
