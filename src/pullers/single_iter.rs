use crate::{
    enumeration::{Element, Enumeration, Regular},
    ConcurrentIter,
};
use core::marker::PhantomData;

pub struct SingleIter<'a, I, E = Regular>
where
    I: ConcurrentIter<E>,
    E: Enumeration,
{
    con_iter: &'a I,
    phantom: PhantomData<E>,
}

impl<'a, I, E> SingleIter<'a, I, E>
where
    I: ConcurrentIter<E>,
    E: Enumeration,
{
    pub fn new(con_iter: &'a I) -> Self {
        Self {
            con_iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, E> Iterator for SingleIter<'a, I, E>
where
    I: ConcurrentIter<E>,
    E: Enumeration,
{
    type Item = <E::Element as Element>::ElemOf<I::Item>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next()
    }
}
