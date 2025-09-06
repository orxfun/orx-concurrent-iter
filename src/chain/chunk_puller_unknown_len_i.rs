use crate::{
    ChunkPuller, ConcurrentIter,
    chain::{chunk::ChunkOfEither, con_iter_unknown_len_i::ChainUnknownLenI},
};
use std::sync::atomic::Ordering;

pub struct ChainedChunkPullerUnknownLenI<'i, I, J>
where
    I: ConcurrentIter + 'i,
    J: ConcurrentIter<Item = I::Item> + 'i,
{
    chain: &'i ChainUnknownLenI<I, J>,
    p: I::ChunkPuller<'i>,
    q: J::ChunkPuller<'i>,
    p_consumed: bool,
}

impl<'i, I, J> ChainedChunkPullerUnknownLenI<'i, I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(super) fn new(chain: &'i ChainUnknownLenI<I, J>, chunk_size: usize) -> Self {
        let p = chain.i.chunk_puller(chunk_size);
        let q = chain.j.chunk_puller(chunk_size);
        Self {
            chain,
            p,
            q,
            p_consumed: false,
        }
    }
}

impl<'i, I, J> ChunkPuller for ChainedChunkPullerUnknownLenI<'i, I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    type ChunkItem = I::Item;

    type Chunk<'c>
        = ChunkOfEither<
        <I::ChunkPuller<'i> as ChunkPuller>::Chunk<'c>,
        <J::ChunkPuller<'i> as ChunkPuller>::Chunk<'c>,
    >
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        // p and q have the same chunk size
        self.p.chunk_size()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        match self.p_consumed {
            false => match self.p.pull() {
                Some(p) => {
                    _ = self
                        .chain
                        .num_pulled_i
                        .fetch_add(p.len(), Ordering::Relaxed);
                    Some(ChunkOfEither::P(p))
                }
                None => {
                    self.p_consumed = true;
                    self.q.pull().map(ChunkOfEither::Q)
                }
            },
            true => self.q.pull().map(ChunkOfEither::Q),
        }
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        match self.p_consumed {
            false => match self.p.pull_with_idx() {
                Some((idx, p)) => {
                    _ = self
                        .chain
                        .num_pulled_i
                        .fetch_add(p.len(), Ordering::Relaxed);
                    Some((idx, ChunkOfEither::P(p)))
                }
                None => {
                    self.p_consumed = true;
                    self.q
                        .pull_with_idx()
                        .map(|(idx, q)| (self.chain.num_pulled_i() + idx, ChunkOfEither::Q(q)))
                }
            },
            true => self
                .q
                .pull_with_idx()
                .map(|(idx, q)| (self.chain.num_pulled_i() + idx, ChunkOfEither::Q(q))),
        }
    }
}
