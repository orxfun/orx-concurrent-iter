use crate::{
    cloned::ConIterCloned,
    copied::ConIterCopied,
    enumerate::Enumerate,
    pullers::{ChunkPuller, EnumeratedItemPuller, ItemPuller},
};

pub trait ConcurrentIter: Send + Sync {
    type Item: Send + Sync;

    type SequentialIter: Iterator<Item = Self::Item>;

    type ChunkPuller<'i>: ChunkPuller<ChunkItem = Self::Item>
    where
        Self: 'i;

    // transform

    fn into_seq_iter(self) -> Self::SequentialIter;

    // iterate

    fn skip_to_end(&self);

    fn next(&self) -> Option<Self::Item>;

    fn next_with_idx(&self) -> Option<(usize, Self::Item)>;

    // len

    fn size_hint(&self) -> (usize, Option<usize>);

    fn try_get_len(&self) -> Option<usize> {
        match self.size_hint() {
            (x, Some(y)) if x == y => Some(x),
            _ => None,
        }
    }

    // pullers

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_>;

    fn item_puller(&self) -> ItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    fn item_puller_with_idx(&self) -> EnumeratedItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    // provided transformations

    fn copied<'a, T>(self) -> ConIterCopied<'a, Self, T>
    where
        T: Send + Sync + Copy,
        Self: ConcurrentIter<Item = &'a T> + Sized,
    {
        ConIterCopied::new(self)
    }

    fn cloned<'a, T>(self) -> ConIterCloned<'a, Self, T>
    where
        T: Send + Sync + Clone,
        Self: ConcurrentIter<Item = &'a T> + Sized,
    {
        ConIterCloned::new(self)
    }

    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }
}
