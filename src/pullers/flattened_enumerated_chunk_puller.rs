use super::ChunkPuller;
use core::iter::Enumerate;

pub struct FlattenedEnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    puller: P,
    current_begin_idx: usize,
    current_chunk: Enumerate<P::Chunk>,
}

impl<P> From<P> for FlattenedEnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    fn from(puller: P) -> Self {
        Self {
            puller,
            current_begin_idx: 0,
            current_chunk: Default::default(),
        }
    }
}

impl<P> FlattenedEnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    pub fn into_chunk_puller(self) -> P {
        self.puller
    }

    fn next_chunk(&mut self) -> Option<(usize, P::ChunkItem)> {
        match self.puller.pull_with_idx() {
            Some((begin_idx, chunk)) => {
                self.current_begin_idx = begin_idx;
                self.current_chunk = chunk.enumerate();
                self.next()
            }
            None => None,
        }
    }
}

impl<P> Iterator for FlattenedEnumeratedChunkPuller<P>
where
    P: ChunkPuller,
{
    type Item = (usize, P::ChunkItem);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current_chunk.next();
        match next.is_some() {
            true => next,
            false => self.next_chunk(),
        }
    }
}
