use crate::{chunk_puller::ChunkPuller, next::NextKind, Enumerated, Regular};
use core::marker::PhantomData;

pub struct ChunksIter<C, K = Regular>
where
    C: ChunkPuller<Enumerated>,
    K: NextKind,
{
    puller: C,
    begin_idx: usize,
    current_chunk: K::SeqIterKind<C::Iter>,
    phantom: PhantomData<K>,
}

impl<C, K> ChunksIter<C, K>
where
    C: ChunkPuller<Enumerated>,
    K: NextKind,
{
    fn next_chunk(&mut self) -> Option<K::Next<C::Item>> {
        match self.puller.pull() {
            Some((begin_idx, chunk)) => {
                self.begin_idx = begin_idx;
                self.current_chunk = K::new_seq_iter(chunk);
                self.next()
            }
            None => None,
        }
    }
}

impl<C, K> Iterator for ChunksIter<C, K>
where
    C: ChunkPuller<Enumerated>,
    K: NextKind,
{
    type Item = K::Next<C::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = K::seq_iter_next(self.begin_idx, &mut self.current_chunk);
        match next.is_some() {
            true => next,
            false => self.next_chunk(),
        }
    }
}
