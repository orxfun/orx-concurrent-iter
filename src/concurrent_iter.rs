use crate::pullers::{ChunkPuller, EnumeratedItemPuller, ItemPuller};

pub trait ConcurrentIter {
    type Item: Send + Sync;

    type SequentialIter: Iterator<Item = Self::Item>;

    type ChunkPuller<'i>: ChunkPuller<ChunkItem = Self::Item> + From<(&'i Self, usize)>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter;

    // iterate

    fn skip_to_end(&self);

    fn next(&self) -> Option<Self::Item>;

    fn next_with_idx(&self) -> Option<(usize, Self::Item)>;

    // len

    fn size_hint(&self) -> (usize, Option<usize>);

    fn try_get_len(&self) -> Option<usize> {
        match self.size_hint() {
            (_, None) => None,
            (_, Some(upper)) => Some(upper),
        }
    }

    // pullers

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        (self, chunk_size).into()
    }

    fn item_puller(&self) -> ItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    fn enumerated_item_puller(&self) -> EnumeratedItemPuller<'_, Self>
    where
        Self: Sized,
    {
        self.into()
    }
}
