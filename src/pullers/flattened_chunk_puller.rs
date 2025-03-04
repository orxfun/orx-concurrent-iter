use crate::chunk_puller::ChunkPuller;

pub struct FlattenedChunkPuller<P>
where
    P: ChunkPuller,
{
    puller: P,
    current_chunk: P::Chunk,
}

impl<P> FlattenedChunkPuller<P>
where
    P: ChunkPuller,
{
    pub(crate) fn new(puller: P) -> Self {
        Self {
            puller,
            current_chunk: Default::default(),
        }
    }

    pub fn into_chunk_puller(self) -> P {
        self.puller
    }

    fn next_chunk(&mut self) -> Option<P::ChunkItem> {
        match self.puller.pull() {
            Some(chunk) => {
                self.current_chunk = chunk;
                self.next()
            }
            None => None,
        }
    }
}

impl<P> Iterator for FlattenedChunkPuller<P>
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
