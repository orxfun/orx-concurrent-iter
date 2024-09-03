use crate::{ConcurrentIter, ConcurrentIterX, NextChunk};

pub trait BufferedChunk<T: Send + Sync> {
    type ConIter: ConcurrentIter<Item = T>;

    fn new(chunk_size: usize) -> Self;

    fn chunk_size(&self) -> usize;

    fn pull(
        &mut self,
        iter: &Self::ConIter,
    ) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>>;
}

pub trait BufferedChunkX<T: Send + Sync> {
    type ConIter: ConcurrentIterX<Item = T>;

    fn new(chunk_size: usize) -> Self;

    fn chunk_size(&self) -> usize;

    fn pull(&mut self, iter: &Self::ConIter) -> Option<impl ExactSizeIterator<Item = T>>;
}
