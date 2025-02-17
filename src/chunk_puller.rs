use crate::next::{NextKind, Regular};

pub trait ChunkPuller<K: NextKind = Regular> {
    type Item;

    type Iter: ExactSizeIterator<Item = Self::Item> + Default;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self) -> Option<K::Next<Self::Iter>>;
}
