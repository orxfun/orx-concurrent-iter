use crate::{chunk_puller::ChunkPuller, next::NextKind, Regular};
use core::marker::PhantomData;

pub struct ChunksIter<C, K = Regular>
where
    C: ChunkPuller<K>,
    K: NextKind,
{
    puller: C,
    begin_idx: K::BeginIdx,
    current_chunk: K::SeqChunkIter<C::Iter>,
    phantom: PhantomData<K>,
}

impl<C, K> ChunksIter<C, K>
where
    C: ChunkPuller<K>,
    K: NextKind,
{
    pub(crate) fn new(puller: C) -> Self {
        Self {
            puller,
            begin_idx: Default::default(),
            current_chunk: Default::default(),
            phantom: PhantomData,
        }
    }

    fn next_chunk(&mut self) -> Option<K::Next<C::ChunkItem>> {
        match self.puller.next().map(K::destruct_chunk) {
            Some((begin_idx, chunk)) => {
                self.begin_idx = begin_idx;
                self.current_chunk = K::into_seq_chunk_iter(chunk);
                self.next()
            }
            None => None,
        }
    }
}

impl<C, K> Iterator for ChunksIter<C, K>
where
    C: ChunkPuller<K>,
    K: NextKind,
{
    type Item = K::Next<C::ChunkItem>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = K::seq_chunk_iter_next(self.begin_idx, &mut self.current_chunk);
        match next.is_some() {
            true => next,
            false => self.next_chunk(),
        }
    }
}
