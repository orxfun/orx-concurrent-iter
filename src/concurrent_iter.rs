use crate::{
    chunk_puller::ChunkPuller,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
};

pub trait ConcurrentIter<E: Enumeration = Regular>: Default {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    type ChunkPuller<'i>: ChunkPuller<E, ChunkItem = Self::Item>
    where
        Self: 'i;

    type EnumerationOf<E2>: ConcurrentIter<E2, Item = Self::Item>
    where
        E2: Enumeration;

    type Regular: ConcurrentIter<Regular, Item = Self::Item>;

    type Enumerated: ConcurrentIter<Enumerated, Item = Self::Item>;

    // into

    fn into_seq_iter(self) -> Self::SeqIter;

    fn into_enumeration_of<E2: Enumeration>(self) -> Self::EnumerationOf<E2>;

    fn enumerated(self) -> Self::Enumerated
    where
        E: IsNotEnumerated;

    fn not_enumerated(self) -> Self::Regular
    where
        E: IsEnumerated;

    // iter

    fn skip_to_end(&self);

    fn next(&self) -> Option<<E::Element as Element>::ElemOf<Self::Item>>;

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_>;

    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<<E::Element as Element>::IterOf<<Self::ChunkPuller<'_> as ChunkPuller<E>>::Iter>>
    {
        self.chunks_iter(chunk_size).next()
    }
}
