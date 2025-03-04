use crate::pullers::ChunkPuller;
use core::iter::Copied;

pub struct CopiedChunkPuller<'i, T, P>
where
    T: Copy + 'i,
    P: ChunkPuller<ChunkItem = &'i T>,
{
    puller: P,
}

impl<'i, T, P> From<P> for CopiedChunkPuller<'i, T, P>
where
    T: Copy + 'i,
    P: ChunkPuller<ChunkItem = &'i T>,
{
    fn from(puller: P) -> Self {
        Self { puller }
    }
}

impl<'i, T, P> ChunkPuller for CopiedChunkPuller<'i, T, P>
where
    T: Copy + 'i,
    P: ChunkPuller<ChunkItem = &'i T>,
{
    type ChunkItem = T;

    type Chunk = Copied<P::Chunk>;

    fn chunk_size(&self) -> usize {
        self.puller.chunk_size()
    }

    fn pull(&mut self) -> Option<Self::Chunk> {
        self.puller.pull().map(|x| x.copied())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| (begin_idx, x.copied()))
    }
}
