use super::ChunkPuller;
use core::iter::Enumerate;

pub struct FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    puller: P,
    current_begin_idx: usize,
    current_chunk: Enumerate<P::Chunk<'c>>,
}

impl<'c, P> From<P> for FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    fn from(puller: P) -> Self {
        Self {
            puller,
            current_begin_idx: 0,
            current_chunk: Default::default(),
        }
    }
}

impl<'c, P> FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    pub fn into_chunk_puller(self) -> P {
        self.puller
    }

    fn next_chunk(&mut self) -> Option<(usize, P::ChunkItem)> {
        let puller = unsafe { &mut *(&mut self.puller as *mut P) };
        match puller.pull_with_idx() {
            Some((begin_idx, chunk)) => {
                self.current_begin_idx = begin_idx;
                self.current_chunk = chunk.enumerate();
                self.next()
            }
            None => None,
        }
    }
}

impl<'c, P> Iterator for FlattenedEnumeratedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    type Item = (usize, P::ChunkItem);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_chunk.next() {
            Some((i, x)) => Some((self.current_begin_idx + i, x)),
            None => self.next_chunk(),
        }
    }
}
