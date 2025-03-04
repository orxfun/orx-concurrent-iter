pub trait ChunkPuller {
    type ChunkItem;

    type Chunk: ExactSizeIterator<Item = Self::ChunkItem>;

    fn pull(&mut self) -> Option<Self::Chunk>;

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)>;
}
