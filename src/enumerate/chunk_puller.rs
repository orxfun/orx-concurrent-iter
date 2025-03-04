use crate::ChunkPuller;

pub struct EnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    puller: P,
}

impl<P> EnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    pub(crate) fn new(puller: P) -> Self {
        Self { puller }
    }
}

impl<P> ChunkPuller for EnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    type ChunkItem = (usize, P::ChunkItem);

    type Chunk = EnumeratedChunk<P::Chunk>;

    fn chunk_size(&self) -> usize {
        self.puller.chunk_size()
    }

    fn pull(&mut self) -> Option<Self::Chunk> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| EnumeratedChunk {
                begin_idx,
                chunk: x.enumerate(),
            })
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk)> {
        self.puller.pull_with_idx().map(|(begin_idx, x)| {
            (
                begin_idx,
                EnumeratedChunk {
                    begin_idx,
                    chunk: x.enumerate(),
                },
            )
        })
    }
}

pub struct EnumeratedChunk<I>
where
    I: Iterator,
{
    chunk: core::iter::Enumerate<I>,
    begin_idx: usize,
}

impl<I> Default for EnumeratedChunk<I>
where
    I: Iterator,
{
    fn default() -> Self {
        todo!()
    }
}

impl<I> Iterator for EnumeratedChunk<I>
where
    I: Iterator,
{
    type Item = (usize, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.chunk.next().map(|(i, x)| (self.begin_idx + i, x))
    }
}

impl<I> ExactSizeIterator for EnumeratedChunk<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.chunk.len()
    }
}
