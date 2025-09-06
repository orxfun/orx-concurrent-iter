use crate::{ChunkPuller, ConcurrentIter};

pub struct ChainedChunkPuller<'i, I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    i: &'i I,
    j: &'i J,
    p: I::ChunkPuller<'i>,
    q: J::ChunkPuller<'i>,
    p_consumed: bool,
}

impl<'i, I, J> ChainedChunkPuller<'i, I, J>
where
    I: ConcurrentIter,
    J: ConcurrentIter<Item = I::Item>,
{
    pub(super) fn new(i: &'i I, j: &'i J, chunk_size: usize) -> Self {
        let p = i.chunk_puller(chunk_size);
        let q = j.chunk_puller(chunk_size);
        Self {
            i,
            j,
            p,
            q,
            p_consumed: false,
        }
    }
}

impl<'i, I, J> ChunkPuller for ChainedChunkPuller<'i, I, J>
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
                .map(|(idx, q)| (idx, ChunkOfEither::Q(q))),
        }
    }
}

pub enum ChunkOfEither<P, Q>
where
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    P(P),
    Q(Q),
}

impl<P, Q> Default for ChunkOfEither<P, Q>
where
    P: ExactSizeIterator + Default,
    Q: ExactSizeIterator<Item = P::Item>,
{
    fn default() -> Self {
        Self::P(Default::default())
    }
}

impl<P, Q> Iterator for ChunkOfEither<P, Q>
where
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    type Item = P::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::P(p) => p.next(),
            Self::Q(q) => q.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::P(p) => p.size_hint(),
            Self::Q(q) => q.size_hint(),
        }
    }
}

impl<P, Q> ExactSizeIterator for ChunkOfEither<P, Q>
where
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    fn len(&self) -> usize {
        match self {
            Self::P(p) => p.len(),
            Self::Q(q) => q.len(),
        }
    }
}
