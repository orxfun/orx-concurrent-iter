use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    chain::chunk_puller_known_len_i::ChainedChunkPullerKnownLenI,
};

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
        = ChainedChunkPullerKnownLenI<'i, I, J>
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
        let (l1, u1) = self.i.size_hint();
        let (l2, u2) = self.j.size_hint();
        match (u1, u2) {
            (Some(u1), Some(u2)) => (l1 + l2, Some(u1 + u2)),
            _ => (l1 + l2, None),
        }
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        ChainedChunkPullerKnownLenI::new(&self.i, &self.j, chunk_size, self.len_i)
    }
}

impl<I, J> ExactSizeConcurrentIter for ChainKnownLenI<I, J>
where
    I: ExactSizeConcurrentIter,
    J: ExactSizeConcurrentIter<Item = I::Item>,
{
    fn len(&self) -> usize {
        self.i.len() + self.j.len()
    }
}
