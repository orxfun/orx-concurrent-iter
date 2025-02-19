use crate::{
    chunks_iter::ChunksIter,
    enumeration::{Element, Enumeration, Regular},
};
use core::marker::PhantomData;

pub trait ChunkPuller<E: Enumeration = Regular>:
    Sized + Iterator<Item = <E::Element as Element>::IterOf<Self::Iter>>
{
    type ChunkItem: Send + Sync;

    type Iter: ExactSizeIterator<Item = Self::ChunkItem> + Default;

    fn chunk_size(&self) -> usize;

    fn flattened(self) -> ChunksIter<Self, E> {
        ChunksIter::new(self)
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
impl<E, T> Iterator for DoNothingChunkPuller<E, T>
where
    E: Enumeration,
    T: Send + Sync,
{
    type Item = <E::Element as Element>::IterOf<<Self as ChunkPuller<E>>::Iter>;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
impl<E, T> ChunkPuller<E> for DoNothingChunkPuller<E, T>
where
    E: Enumeration,
    T: Send + Sync,
{
    type ChunkItem = T;

    type Iter = core::iter::Empty<T>;

    fn chunk_size(&self) -> usize {
        todo!()
    }
}
