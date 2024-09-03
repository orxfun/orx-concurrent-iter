use crate::NextChunk;

pub trait BufferedChunk<T: Send + Sync>: BufferedChunkX<T> {
    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>>;
}

pub trait BufferedChunkX<T: Send + Sync> {
    type ConIter;

    fn new(chunk_size: usize) -> Self;

    fn chunk_size(&self) -> usize;

    fn pull_x(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = T>>;
}
