use crate::{ConcurrentIter, chain::chunk_puller::ChainedChunkPuller};

pub struct Chain<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    i: I,
    j: J,
}

impl<I, J> Chain<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(crate) fn new(i: I, j: J) -> Self {
        Self { i, j }
    }
}

impl<I, J> ConcurrentIter for Chain<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    type Item = I::Item;

    type SequentialIter = core::iter::Chain<I::SequentialIter, J::SequentialIter>;

    type ChunkPuller<'i>
        = ChainedChunkPuller<I::ChunkPuller<'i>, J::ChunkPuller<'i>>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        todo!()
    }

    fn skip_to_end(&self) {
        todo!()
    }

    fn next(&self) -> Option<Self::Item> {
        todo!()
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
