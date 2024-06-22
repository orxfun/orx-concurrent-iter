use crate::iter::atomic_iter::AtomicIter;

pub trait BufferedChunk<T: Send + Sync> {
    type ConIter: AtomicIter<T>;

    fn new(chunk_size: usize) -> Self;

    fn chunk_size(&self) -> usize;

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = T>>;
}
