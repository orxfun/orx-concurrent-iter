use crate::ChunkPuller;

/// Chunk puller of an enumerated concurrent iterator; i.e., [`Enumerate`]
///
/// [`Enumerate`]: crate::enumerate::Enumerate
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

    type Chunk<'c>
        = EnumeratedChunk<P::Chunk<'c>>
    where
        Self: 'c;

    fn chunk_size(&self) -> usize {
        self.puller.chunk_size()
    }

    fn pull(&mut self) -> Option<Self::Chunk<'_>> {
        self.puller
            .pull_with_idx()
            .map(|(begin_idx, x)| EnumeratedChunk {
                begin_idx,
                chunk: x.enumerate(),
            })
    }

    fn pull_with_idx(&mut self) -> Option<(usize, Self::Chunk<'_>)> {
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
    I: ExactSizeIterator + Default,
{
    chunk: core::iter::Enumerate<I>,
    begin_idx: usize,
}

impl<I> Default for EnumeratedChunk<I>
where
    I: ExactSizeIterator + Default,
{
    fn default() -> Self {
        Self {
            chunk: Default::default(),
            begin_idx: 0,
        }
    }
}

impl<I> Iterator for EnumeratedChunk<I>
where
    I: ExactSizeIterator + Default,
{
    type Item = (usize, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.chunk.next().map(|(i, x)| (self.begin_idx + i, x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.chunk.len();
        (len, Some(len))
    }
}

impl<I> ExactSizeIterator for EnumeratedChunk<I>
where
    I: ExactSizeIterator + Default,
{
    fn len(&self) -> usize {
        self.chunk.len()
    }
}
