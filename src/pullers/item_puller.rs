use crate::{
    enumeration::{Element, Enumeration, Regular},
    ConcurrentIter,
};
use core::marker::PhantomData;

pub struct ItemPuller<'a, I, E = Regular>
where
    I: ConcurrentIter<E>,
    E: Enumeration,
{
    con_iter: &'a I,
    phantom: PhantomData<E>,
}

impl<'a, I, E> ItemPuller<'a, I, E>
where
    I: ConcurrentIter<E>,
    E: Enumeration,
{
    pub(crate) fn new(con_iter: &'a I) -> Self {
        Self {
            con_iter,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn pull(&mut self) -> Option<<E::Element as Element>::ElemOf<I::Item>> {
        self.next()
    }
}

impl<I, E> Iterator for ItemPuller<'_, I, E>
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
