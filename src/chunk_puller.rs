use crate::{
    chunks_iter::ChunksIter,
    next::{NextKind, Regular},
};

pub trait ChunkPuller<K: NextKind = Regular>: Sized {
    type Item: Send + Sync;

    type Iter: ExactSizeIterator<Item = Self::Item> + Default;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self) -> Option<K::NextChunk<Self::Item, Self::Iter>>;

    fn flatten(self) -> ChunksIter<Self, K> {
        ChunksIter::new(self)
    }
}
