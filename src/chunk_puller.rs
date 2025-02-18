use crate::{
    chunks_iter::ChunksIter,
    next::{NextKind, Regular},
};

pub trait ChunkPuller<K: NextKind = Regular>:
    Sized + Iterator<Item = K::NextChunk<Self::ChunkItem, Self::Iter>>
{
    type ChunkItem: Send + Sync;

    type Iter: ExactSizeIterator<Item = Self::ChunkItem> + Default;

    fn chunk_size(&self) -> usize;

    fn flatten(self) -> ChunksIter<Self, K> {
        ChunksIter::new(self)
    }
}
