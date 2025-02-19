use super::con_iter_cloned::ConIterCloned;
use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Enumeration, Regular},
};

pub trait IntoClonedConcurrentIter<'a, T, E = Regular>
where
    E: Enumeration,
    T: Send + Sync + Clone + 'a,
    Self: ConcurrentIter<E, Item = &'a T>,
{
    fn cloned(self) -> ConIterCloned<'a, Self, T, E>;
}

impl<'a, T, E, I> IntoClonedConcurrentIter<'a, T, E> for I
where
    E: Enumeration,
    T: Send + Sync + Clone + 'a,
    I: ConcurrentIter<E, Item = &'a T>,
{
    fn cloned(self) -> ConIterCloned<'a, Self, T, E> {
        ConIterCloned::new(self)
    }
}
