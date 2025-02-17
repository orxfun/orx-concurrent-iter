use crate::{
    chunks_iter::ChunksIter,
    next::{NextKind, Regular},
};

pub trait ConcurrentIter<K: NextKind = Regular> {
    /// Type of the items that the iterator yields.
    type Item;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    type ChunksIter<'i>: ChunksIter<K, Item = Self::Item>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SeqIter;

    fn skip_to_end(&self);

    fn next(&self) -> Option<K::Next<Self::Item>>;

    fn next_chunk<'i>(
        &'i self,
        chunk_size: usize,
    ) -> Option<K::Next<<Self::ChunksIter<'i> as ChunksIter<K>>::Iter>> {
        self.in_chunks(chunk_size).pull()
    }

    fn in_chunks(&self, chunk_size: usize) -> Self::ChunksIter<'_>;
}
