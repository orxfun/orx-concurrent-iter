use crate::enumeration::{Element, Enumeration, Regular};

pub trait SinglePuller<E: Enumeration = Regular>: Sized {
    type ChunkItem: Send + Sync;

    type Iter<'c>: ExactSizeIterator<Item = Self::ChunkItem> + Default
    where
        Self: 'c;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self) -> Option<<E::Element as Element>::IterOf<Self::Iter<'_>>>;
}
