use crate::{
    chunk_puller::DoNothingChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Enumerated, Enumeration, Regular},
};
use core::marker::PhantomData;

pub struct Cloned<'a, I, T, E = Regular>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T>,
{
    con_iter: I,
    phantom: PhantomData<&'a (T, E)>,
}

impl<'a, I, T, E> Default for Cloned<'a, I, T, E>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T>,
{
    fn default() -> Self {
        Self {
            con_iter: I::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, I, T, E> Cloned<'a, I, T, E>
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

impl<'a, I, T, E> ConcurrentIter<E> for Cloned<'a, I, T, E>
where
    E: Enumeration,
    T: Send + Sync + Clone,
    I: ConcurrentIter<E, Item = &'a T>,
{
    type Item = T;

    type SeqIter = core::iter::Cloned<I::SeqIter>;

    type ChunkPuller<'i>
        = DoNothingChunkPuller<E, T>
    where
        Self: 'i;

    type Regular = Cloned<'a, I::Regular, T, Regular>;

    type Enumerated = Cloned<'a, I::Enumerated, T, Enumerated>;

    fn into_seq_iter(self) -> Self::SeqIter {
        self.con_iter.into_seq_iter().cloned()
    }

    fn enumerated(self) -> Self::Enumerated
    where
        E: crate::enumeration::IsNotEnumerated,
    {
        todo!()
    }

    fn not_enumerated(self) -> Self::Regular
    where
        E: crate::enumeration::IsEnumerated,
    {
        todo!()
    }

    fn skip_to_end(&self) {
        todo!()
    }

    fn next(
        &self,
    ) -> Option<
        <<E as crate::enumeration::Enumeration>::Element as crate::enumeration::Element>::ElemOf<
            Self::Item,
        >,
    > {
        todo!()
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
