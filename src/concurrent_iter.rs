use crate::{
    chunk_puller::ChunkPuller,
    next::{NextKind, Regular},
    Enumerated,
};

pub trait ConcurrentIter<K: NextKind = Regular>: Default {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    type ChunkPuller<'i>: ChunkPuller<K, Item = Self::Item>
    where
        Self: 'i;

    type Regular: ConcurrentIter<Regular, Item = Self::Item>;

    type Enumerated: ConcurrentIter<Enumerated, Item = Self::Item>;

    // into

    fn into_seq_iter(self) -> Self::SeqIter;

    // enumeration

    fn as_enumerated(&self) -> Self::Enumerated;

    // iter

    fn skip_to_end(&self);

    fn next(&self) -> Option<K::Next<Self::Item>>;

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_>;

    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<K::NextChunk<Self::Item, <Self::ChunkPuller<'_> as ChunkPuller<K>>::Iter>> {
        self.chunks_iter(chunk_size).pull()
    }
}
