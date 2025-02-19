use crate::{
    chunk_puller::ChunkPuller,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
};

pub trait ConcurrentIterEnum<E: Enumeration, T>: Default {
    type EnumerationOf<E2>: ConcurrentIter<E2, Item = T>
    where
        E2: Enumeration;

    fn into_enumeration_of<E2: Enumeration>(self) -> Self::EnumerationOf<E2>;
}

pub trait ConcurrentIter<E: Enumeration = Regular>: ConcurrentIterEnum<E, Self::Item> {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Inner type which is the source of the data to be iterated, which in addition is a regular sequential `Iterator`.
    type SeqIter: Iterator<Item = Self::Item>;

    type ChunkPuller<'i>: ChunkPuller<E, ChunkItem = Self::Item>
    where
        Self: 'i;

    // into

    fn into_seq_iter(self) -> Self::SeqIter;

    fn enumerated(self) -> Self::EnumerationOf<Enumerated>
    where
        E: IsNotEnumerated,
    {
        self.into_enumeration_of()
    }

    fn not_enumerated(self) -> Self::EnumerationOf<Regular>
    where
        E: IsEnumerated,
    {
        self.into_enumeration_of()
    }

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
