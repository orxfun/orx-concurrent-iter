use super::ChunkPuller;

pub struct FlattenedChunkPuller<'c, P>
where
    P: ChunkPuller + 'c,
{
    puller: P,
    current_chunk: P::Chunk<'c>,
}

impl<'c, P> From<P> for FlattenedChunkPuller<'c, P>
where
    P: ChunkPuller,
{
    fn from(puller: P) -> Self {
        Self {
            puller,
            current_chunk: Default::default(),
        }
    }
}

impl<'c, P> FlattenedChunkPuller<'c, P>
where
    P: ChunkPuller,
{
    pub fn into_chunk_puller(self) -> P {
        self.puller
    }

    fn next_chunk(&mut self) -> Option<P::ChunkItem> {
        let puller = unsafe { &mut *(&mut self.puller as *mut P) };
        match puller.pull() {
            Some(chunk) => {
                self.current_chunk = chunk;
                self.next()
            }
            None => None,
        }
    }
}

impl<'c, P> Iterator for FlattenedChunkPuller<'c, P>
where
    P: ChunkPuller,
{
    type Item = P::ChunkItem;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current_chunk.next();
        match next.is_some() {
            true => next,
            false => self.next_chunk(),
        }
    }
}
