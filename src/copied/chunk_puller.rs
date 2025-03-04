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

    type Chunk<'c>
        = Copied<P::Chunk<'c>>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.puller.chunk_size()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.puller.pull().map(|x| x.copied())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| (begin_idx, x.copied()))
    }
}
