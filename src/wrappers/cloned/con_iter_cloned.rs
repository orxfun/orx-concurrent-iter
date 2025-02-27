use super::chunk_puller_cloned::ChunkPullerCloned;
use crate::{
    concurrent_iter::{ConcurrentIter, ConcurrentIterEnum},
    enumeration::{Element, Enumeration, Regular},
};
use core::{iter::Cloned, marker::PhantomData};

pub struct ConIterCloned<'a, I, T, E = Regular>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T>,
{
    con_iter: I,
    phantom: PhantomData<&'a (T, E)>,
}

impl<'a, I, T, E> Default for ConIterCloned<'a, I, T, E>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T> + Default,
{
    fn default() -> Self {
        Self {
            con_iter: I::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, E> ConIterCloned<'a, I, T, E>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T>,
{
    pub(super) fn new(con_iter: I) -> Self {
        Self {
            con_iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, E> ConcurrentIterEnum<E, T> for ConIterCloned<'a, I, T, E>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T> + ConcurrentIterEnum<E, &'a T>,
{
    type EnumerationOf<E2>
        = ConIterCloned<'a, <I as ConcurrentIterEnum<E, &'a T>>::EnumerationOf<E2>, T, E2>
    where
        E2: Enumeration;

    fn into_enumeration_of<E2: Enumeration>(self) -> Self::EnumerationOf<E2> {
        ConIterCloned::new(self.con_iter.into_enumeration_of())
    }
}

impl<'a, I, T, E> ConcurrentIter<E> for ConIterCloned<'a, I, T, E>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T>,
{
    type Item = T;

    type SeqIter = Cloned<I::SeqIter>;

    type ChunkPuller<'i>
        = ChunkPullerCloned<'a, T, E, I::ChunkPuller<'i>>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.con_iter.into_seq_iter().cloned()
    }

    fn skip_to_end(&self) {
        self.con_iter.skip_to_end()
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        self.con_iter
            .next()
            .map(<<E as Enumeration>::Element as Element>::cloned_elem)
    }

    fn chunks_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self.con_iter.chunks_puller(chunk_size))
    }
}
