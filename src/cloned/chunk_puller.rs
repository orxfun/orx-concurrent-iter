use crate::pullers::ChunkPuller;
use core::iter::Cloned;

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

    type Chunk = Cloned<P::Chunk>;

    fn pull(&mut self) -> Option<Self::Chunk> {
        self.puller.pull().map(|x| x.cloned())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| (begin_idx, x.cloned()))
    }
}
