use crate::chunk_puller::ChunkPuller;
use crate::enumeration::{Element, Enumeration};
use core::iter::Cloned;
use core::marker::PhantomData;

pub struct ChunksIterCloned<'a, T, E, C>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    C: ChunkPuller<E, ChunkItem = &'a T>,
{
    chunks_iter: C,
    phantom: PhantomData<&'a (T, E)>,
}

impl<'a, T, E, C> ChunksIterCloned<'a, T, E, C>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    C: ChunkPuller<E, ChunkItem = &'a T>,
{
    pub(super) fn new(chunks_iter: C) -> Self {
        Self {
            chunks_iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, E, C> ChunkPuller<E> for ChunksIterCloned<'a, T, E, C>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    C: ChunkPuller<E, ChunkItem = &'a T>,
{
    type ChunkItem = T;

    type Iter = Cloned<C::Iter>;

    fn chunk_size(&self) -> usize {
        self.chunks_iter.chunk_size()
    }
}

impl<'a, T, E, C> Iterator for ChunksIterCloned<'a, T, E, C>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    C: ChunkPuller<E, ChunkItem = &'a T>,
{
    type Item = <E::Element as Element>::IterOf<<Self as ChunkPuller<E>>::Iter>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks_iter
            .next()
            .map(<<E as Enumeration>::Element as Element>::cloned_iter)
    }
}
