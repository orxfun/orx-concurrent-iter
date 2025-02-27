use crate::pullers::ChunkPuller;
use crate::enumeration::{Element, Enumeration};
use core::iter::Cloned;
use core::marker::PhantomData;

pub struct ChunkPullerCloned<'a, T, E, P>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    P: ChunkPuller<E, ChunkItem = &'a T>,
{
    chunks_puller: P,
    phantom: PhantomData<&'a (T, E)>,
}

impl<'a, T, E, P> ChunkPullerCloned<'a, T, E, P>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    P: ChunkPuller<E, ChunkItem = &'a T>,
{
    pub(super) fn new(chunks_puller: P) -> Self {
        Self {
            chunks_puller,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, E, P> ChunkPuller<E> for ChunkPullerCloned<'a, T, E, P>
where
    T: Send + Sync + Clone + 'a,
    E: Enumeration,
    P: ChunkPuller<E, ChunkItem = &'a T>,
{
    type ChunkItem = T;

    type Iter<'c>
        = Cloned<P::Iter<'c>>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.chunks_puller.chunk_size()
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        self.chunks_puller
            .pull()
            .map(<<E as Enumeration>::Element as Element>::cloned_iter)
    }
}
