use crate::{
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
    pullers::{ChunkPuller, ItemPuller},
};

pub trait ConcurrentIterEnum<E: Enumeration, T> {
    type EnumerationOf<E2>: ConcurrentIter<E2, Item = T>
    where
        E2: Enumeration;

    fn into_enumeration_of<E2: Enumeration>(self) -> Self::EnumerationOf<E2>;
}

pub trait ConcurrentIter<E: Enumeration = Regular>:
    Send + Sync + Sized + ConcurrentIterEnum<E, Self::Item>
{
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
        Self: ConcurrentIterEnum<E, Self::Item>,
    {
        self.into_enumeration_of()
    }

    fn not_enumerated(self) -> Self::EnumerationOf<Regular>
    where
        E: IsEnumerated,
        Self: ConcurrentIterEnum<E, Self::Item>,
    {
        self.into_enumeration_of()
    }

    // iter

    fn skip_to_end(&self);

    fn next(&self) -> Option<<E::Element as Element>::ElemOf<Self::Item>>;

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_>;

    fn item_puller(&self) -> ItemPuller<Self, E> {
        ItemPuller::new(self)
    }

    fn size_hint(&self) -> (usize, Option<usize>);

    fn try_get_len(&self) -> Option<usize> {
        match self.size_hint() {
            (_, None) => None,
            (_, Some(upper)) => Some(upper),
        }
    }
}
