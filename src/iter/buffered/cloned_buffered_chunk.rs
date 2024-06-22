use super::buffered_chunk::BufferedChunk;
use crate::iter::cloned::Cloned;
use std::marker::PhantomData;

pub struct ClonedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Clone,
    C: BufferedChunk<&'a T>,
{
    chunk: C,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, C> BufferedChunk<T> for ClonedBufferedChunk<'a, T, C>
where
    T: Send + Sync + Clone,
    C: BufferedChunk<&'a T>,
{
    type ConIter = Cloned<'a, T, C::ConIter>;

    fn new(chunk_size: usize) -> Self {
        Self {
            chunk: C::new(chunk_size),
            phantom: Default::default(),
        }
    }

    fn chunk_size(&self) -> usize {
        self.chunk.chunk_size()
    }

    fn pull(
        &mut self,
        iter: &Self::ConIter,
        begin_idx: usize,
    ) -> Option<impl ExactSizeIterator<Item = T>> {
        self.chunk
            .pull(iter.underlying_iter(), begin_idx)
            .map(|x| x.cloned())
    }
}
