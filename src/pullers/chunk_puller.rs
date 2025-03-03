use super::{pulled_chunk_iter::PulledChunkIter, FlattenedChunkPuller};
use crate::enumeration::{Element, Enumeration, EnumerationCore, Regular};
use core::marker::PhantomData;

pub trait ChunkPuller<E: Enumeration = Regular>: Sized {
    type ChunkItem: Send + Sync;

    type Iter<'c>: ExactSizeIterator<Item = Self::ChunkItem> + Default
    where
        Self: 'c;

    fn chunk_size(&self) -> usize;

    fn flattened<'c>(self) -> FlattenedChunkPuller<'c, Self, E> {
        FlattenedChunkPuller::new(self)
    }

    fn pull(&mut self) -> Option<<E::Element as Element>::IterOf<Self::Iter<'_>>>;

    fn pulli(&mut self) -> Option<PulledChunkIter<Self::Iter<'_>, E>> {
        self.pull().map(|x| {
            let (begin_idx, chunk) = E::destruct_chunk(x);
            E::new_pulled_chunk_iter(begin_idx, chunk)
        })
    }
}

// dev-only

pub struct DoNothingChunkPuller<E, T>(PhantomData<(E, T)>)
where
    E: Enumeration,
    T: Send + Sync;
impl<E, T> DoNothingChunkPuller<E, T>
where
    E: Enumeration,
    T: Send + Sync,
{
    #[allow(dead_code)]
    pub fn new<X>(_: X, _: usize) -> Self {
        Self(Default::default())
    }
}
impl<E, T> Default for DoNothingChunkPuller<E, T>
where
    E: Enumeration,
    T: Send + Sync,
{
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<E, T> ChunkPuller<E> for DoNothingChunkPuller<E, T>
where
    E: Enumeration,
    T: Send + Sync,
{
    type ChunkItem = T;

    type Iter<'c>
        = core::iter::Empty<T>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        0
    }

    fn pull(&mut self) -> Option<<<E as Enumeration>::Element as Element>::IterOf<Self::Iter<'_>>> {
        None
    }
}
