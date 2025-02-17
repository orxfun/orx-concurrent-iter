use crate::next::{NextKind, Regular};

pub trait ChunksIter<K: NextKind = Regular> {
    type Item;

    type Iter: ExactSizeIterator<Item = Self::Item>;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self) -> Option<K::Next<Self::Iter>>;
}
