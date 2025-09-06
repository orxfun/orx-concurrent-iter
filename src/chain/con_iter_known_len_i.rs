use crate::{ConcurrentIter, chain::chunk_puller::ChainedChunkPuller};

pub struct ChainKnownLenI<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    i: I,
    j: J,
    len_i: usize,
}

impl<I, J> ChainKnownLenI<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(super) fn new(i: I, j: J, len_i: usize) -> Self {
        Self { i, j, len_i }
    }
}

impl<I, J> ConcurrentIter for ChainKnownLenI<I, J>
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
        self.i.skip_to_end();
        self.j.skip_to_end();
    }

    fn next(&self) -> Option<Self::Item> {
        self.i.next().or_else(|| self.j.next())
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.i
            .next_with_idx()
            .or_else(|| self.j.next_with_idx().map(|(idx, x)| (self.len_i + idx, x)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l, u) = self.j.size_hint();
        (self.len_i + l, u.map(|u| self.len_i + u))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        ChainedChunkPuller::new(
            self.i.chunk_puller(chunk_size),
            self.j.chunk_puller(chunk_size),
            false,
        )
    }
}
