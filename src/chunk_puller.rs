use crate::{
    chunks_iter::ChunksIter,
    enumeration::{Element, Enumeration, Regular},
};

pub trait ChunkPuller<K: Enumeration = Regular>:
    Sized + Iterator<Item = <K::Element as Element>::IterOf<Self::Iter>>
{
    type ChunkItem: Send + Sync;

    type Iter: ExactSizeIterator<Item = Self::ChunkItem> + Default;

    fn chunk_size(&self) -> usize;

    fn flattened(self) -> ChunksIter<Self, K> {
        ChunksIter::new(self)
    }
}
