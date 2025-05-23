use crate::pullers::ChunkPuller;
use core::iter::Cloned;

/// Chunk puller of a cloned concurrent iterator; i.e., [`ConIterCloned`]
///
/// [`ConIterCloned`]: crate::cloned::ConIterCloned
pub struct ClonedChunkPuller<'i, T, P>
where
    T: Clone + 'i,
    P: ChunkPuller<ChunkItem = &'i T>,
{
    puller: P,
}

impl<'i, T, P> From<P> for ClonedChunkPuller<'i, T, P>
where
    T: Clone + 'i,
    P: ChunkPuller<ChunkItem = &'i T>,
{
    fn from(puller: P) -> Self {
        Self { puller }
    }
}

impl<'i, T, P> ChunkPuller for ClonedChunkPuller<'i, T, P>
where
    T: Clone + 'i,
    P: ChunkPuller<ChunkItem = &'i T>,
{
    type ChunkItem = T;

    type Chunk<'c>
        = Cloned<P::Chunk<'c>>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.puller.chunk_size()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.puller.pull().map(|x| x.cloned())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| (begin_idx, x.cloned()))
    }
}
