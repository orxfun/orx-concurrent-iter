use crate::{ConcurrentIter, ExactSizeConcurrentIter, chain::chunk_puller::ChainedChunkPuller};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ChainUnknownLenI<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    i: I,
    j: J,
    len_i: AtomicUsize,
}

impl<I, J> ChainUnknownLenI<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(super) fn new(i: I, j: J) -> Self {
        let len_i = 0.into();
        Self { i, j, len_i }
    }
}

impl<I, J> ConcurrentIter for ChainUnknownLenI<I, J>
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
        match self.i.next_with_idx() {
            Some((idx, x)) => {
                _ = self.len_i.fetch_max(idx, Ordering::Relaxed);
                Some(x)
            }
            None => self.j.next(),
        }
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        match self.i.next_with_idx() {
            Some((idx, x)) => {
                _ = self.len_i.fetch_max(idx, Ordering::Relaxed);
                Some((idx, x))
            }
            None => self.j.next_with_idx().map(|(idx, x)| {
                let len_i = self.len_i.load(Ordering::Relaxed);
                (len_i + idx, x)
            }),
        }
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
        ChainedChunkPuller::new(
            self.i.chunk_puller(chunk_size),
            self.j.chunk_puller(chunk_size),
            false,
        )
    }
}

impl<I, J> ExactSizeConcurrentIter for ChainUnknownLenI<I, J>
where
    I: ExactSizeConcurrentIter,
    J: ExactSizeConcurrentIter<Item = I::Item>,
{
    fn len(&self) -> usize {
        self.i.len() + self.j.len()
    }
}
