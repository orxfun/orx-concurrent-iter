use crate::next::Next;

pub trait ChunksIter {
    type Item;

    type Iter: ExactSizeIterator<Item = Self::Item>;

    fn chunk_size(&self) -> usize;

    fn pull<N: Next<Self::Iter>>(&mut self) -> Option<N>;
}
