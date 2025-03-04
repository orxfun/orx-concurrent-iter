use crate::pullers::{FlattenedChunkPuller, FlattenedEnumeratedChunkPuller};

pub trait ChunkPuller {
    type ChunkItem;

    type Chunk: ExactSizeIterator<Item = Self::ChunkItem> + Default;

    fn pull(&mut self) -> Option<Self::Chunk>;

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)>;

    fn flattened(self) -> FlattenedChunkPuller<Self>
    where
        Self: Sized,
    {
        self.into()
    }

    fn flattened_enumerated(self) -> FlattenedEnumeratedChunkPuller<Self>
    where
        Self: Sized,
    {
        self.into()
    }
}
