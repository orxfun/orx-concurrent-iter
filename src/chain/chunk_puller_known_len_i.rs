use crate::{ChunkPuller, ConcurrentIter, chain::chunk::ChunkOfEither};

pub struct ChainedChunkPullerKnownLenI<'i, I, J>
where
    I: ConcurrentIter + 'i,
    J: ConcurrentIter<Item = I::Item> + 'i,
{
    p: I::ChunkPuller<'i>,
    q: J::ChunkPuller<'i>,
    len_i: usize,
    p_consumed: bool,
}

impl<'i, I, J> ChainedChunkPullerKnownLenI<'i, I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(super) fn new(i: &'i I, j: &'i J, chunk_size: usize, len_i: usize) -> Self {
        let p = i.chunk_puller(chunk_size);
        let q = j.chunk_puller(chunk_size);
        Self {
            p,
            q,
            len_i,
            p_consumed: false,
        }
    }
}

impl<'i, I, J> ChunkPuller for ChainedChunkPullerKnownLenI<'i, I, J>
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
            false => {
                let p = self.p.pull().map(ChunkOfEither::P);
                match p.is_some() {
                    true => p,
                    false => {
                        self.p_consumed = true;
                        self.q.pull().map(ChunkOfEither::Q)
                    }
                }
            }
            true => self.q.pull().map(ChunkOfEither::Q),
        }
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        match self.p_consumed {
            false => {
                let p = self
                    .p
                    .pull_with_idx()
                    .map(|(idx, p)| (idx, ChunkOfEither::P(p)));
                match p.is_some() {
                    true => p,
                    false => {
                        self.p_consumed = true;
                        self.q
                            .pull_with_idx()
                            .map(|(idx, q)| (idx, ChunkOfEither::Q(q)))
                    }
                }
            }
            true => self
                .q
                .pull_with_idx()
                .map(|(idx, q)| (self.len_i + idx, ChunkOfEither::Q(q))),
        }
    }
}
