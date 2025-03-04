use crate::pullers::{FlattenedChunkPuller, FlattenedEnumeratedChunkPuller};

pub trait ChunkPuller {
    type ChunkItem;

    type Chunk<'c>: ExactSizeIterator<Item = Self::ChunkItem> + Default
    where
        Self: 'c;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self) -> Option<Self::Chunk<'_>>;

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)>;

    fn flattened<'c>(self) -> FlattenedChunkPuller<'c, Self>
    where
        Self: Sized,
    {
        self.into()
    }

    fn flattened_with_idx<'c>(self) -> FlattenedEnumeratedChunkPuller<'c, Self>
    where
        Self: Sized,
    {
        self.into()
    }
}
