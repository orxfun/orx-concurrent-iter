use crate::ChunkPuller;

pub struct ChainedChunkPuller<P, Q>
where
    P: ChunkPuller,
    Q: ChunkPuller<ChunkItem = P::ChunkItem>,
{
    p: P,
    q: Q,
    p_consumed: bool,
}

impl<P, Q> ChainedChunkPuller<P, Q>
where
    P: ChunkPuller,
    Q: ChunkPuller<ChunkItem = P::ChunkItem>,
{
    pub(crate) fn new(p: P, q: Q) -> Self {
        let p_consumed = false;
        Self { p, q, p_consumed }
    }
}

impl<P, Q> ChunkPuller for ChainedChunkPuller<P, Q>
where
    P: ChunkPuller,
    Q: ChunkPuller<ChunkItem = P::ChunkItem>,
{
    type ChunkItem = P::ChunkItem;

    type Chunk<'c>
        = ChunkOfEither<P::Chunk<'c>, Q::Chunk<'c>>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        match self.p_consumed {
            false => self.p.chunk_size(),
            true => self.q.chunk_size(),
        }
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        match self.p_consumed {
            false => {
                // SAFETY: s will be used to recursively call this function iff pulled chunks is None
                // in which case the mutable reference is not used.
                let s = unsafe { &mut *(self as *mut Self) };
                let p = self.p.pull().map(ChunkOfEither::P);
                match p.is_some() {
                    true => p,
                    false => {
                        self.p_consumed = true;
                        s.pull()
                    }
                }
            }
            true => self.q.pull().map(ChunkOfEither::Q),
        }
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        match self.p_consumed {
            false => {
                // SAFETY: s will be used to recursively call this function iff pulled chunks is None
                // in which case the mutable reference is not used.
                let s = unsafe { &mut *(self as *mut Self) };
                let p = self
                    .p
                    .pull_with_idx()
                    .map(|(idx, p)| (idx, ChunkOfEither::P(p)));
                match p.is_some() {
                    true => p,
                    false => {
                        self.p_consumed = true;
                        s.pull_with_idx()
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
    P: ExactSizeIterator,
    Q: ExactSizeIterator<Item = P::Item>,
{
    fn default() -> Self {
        todo!()
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
