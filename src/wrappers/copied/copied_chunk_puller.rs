use crate::pullers::ChunkPuller;
use core::iter::Copied;

pub struct CopiedChunkPuller<'a, T, P>
where
    T: Copy + 'a,
    P: ChunkPuller<ChunkItem = &'a T>,
{
    puller: P,
}

impl<'a, T, P> From<P> for CopiedChunkPuller<'a, T, P>
where
    T: Copy + 'a,
    P: ChunkPuller<ChunkItem = &'a T>,
{
    fn from(puller: P) -> Self {
        Self { puller }
    }
}

impl<'a, T, P> ChunkPuller for CopiedChunkPuller<'a, T, P>
where
    T: Copy + 'a,
    P: ChunkPuller<ChunkItem = &'a T>,
{
    type ChunkItem = T;

    type Chunk = Copied<P::Chunk>;

    fn pull(&mut self) -> Option<Self::Chunk> {
        self.puller.pull().map(|x| x.copied())
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| (begin_idx, x.copied()))
    }
}
