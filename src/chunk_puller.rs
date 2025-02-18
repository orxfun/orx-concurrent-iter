use crate::{
    chunks_iter::ChunksIter,
    next::{NextKind, Regular},
};

pub trait ChunkPuller<K: NextKind = Regular>: Sized {
    type Item;

    type Iter: ExactSizeIterator<Item = Self::Item> + Default;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self) -> Option<K::Next<Self::Iter>>;

    fn flatten(self) -> ChunksIter<Self, K> {
        ChunksIter::new(self)
    }
}
