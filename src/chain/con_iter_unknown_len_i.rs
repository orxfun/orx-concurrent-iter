use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    chain::chunk_puller_unknown_len_i::ChainedChunkPullerUnknownLenI,
};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Chain of two concurrent iterators where the length of the first iterator is not
/// known with certainly; i.e., `I` does not implement `ExactSizeConcurrentIter`.
pub struct ChainUnknownLenI<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(super) i: I,
    pub(super) j: J,
    pub(super) num_pulled_i: AtomicUsize,
}

impl<I, J> ChainUnknownLenI<I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(crate) fn new(i: I, j: J) -> Self {
        Self {
            i,
            j,
            num_pulled_i: 0.into(),
        }
    }

    #[inline(always)]
    pub(super) fn num_pulled_i(&self) -> usize {
        self.num_pulled_i.load(Ordering::Relaxed)
        // match self.num_pulled_i.load(Ordering::Relaxed) {
        //     0 => 0,
        //     n => n + 1,
        // }
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
        = ChainedChunkPullerUnknownLenI<'i, I, J>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        self.i.into_seq_iter().chain(self.j.into_seq_iter())
    }

    fn skip_to_end(&self) {
        self.i.skip_to_end();
        self.j.skip_to_end();
    }

    fn next(&self) -> Option<Self::Item> {
        match self.i.next() {
            Some(x) => {
                // _ = self.num_pulled_i.fetch_max(idx, Ordering::Relaxed);
                _ = self.num_pulled_i.fetch_add(1, Ordering::Relaxed);
                Some(x)
            }
            None => self.j.next(),
        }
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        match self.i.next_with_idx() {
            Some((idx, x)) => {
                _ = self.num_pulled_i.fetch_add(1, Ordering::Relaxed);
                Some((idx, x))
            }
            None => self
                .j
                .next_with_idx()
                .map(|(idx, x)| (self.num_pulled_i() + idx, x)),
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
        ChainedChunkPullerUnknownLenI::new(self, chunk_size)
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
