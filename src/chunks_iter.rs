use crate::{
    chunk_puller::ChunkPuller,
    enumeration::{Element, Enumeration, Regular},
};
use core::marker::PhantomData;

pub struct ChunksIter<'c, P, K = Regular>
where
    P: ChunkPuller<K> + 'c,
    K: Enumeration,
{
    puller: P,
    begin_idx: K::BeginIdx,
    current_chunk: K::SeqChunkIter<P::Iter<'c>>,
    phantom: PhantomData<K>,
}

impl<'c, P, K> ChunksIter<'c, P, K>
where
    P: ChunkPuller<K> + 'c,
    K: Enumeration,
{
    pub(crate) fn new(puller: P) -> Self {
        Self {
            puller,
            begin_idx: Default::default(),
            current_chunk: Default::default(),
            phantom: PhantomData,
        }
    }

    fn next_chunk(&mut self) -> Option<<K::Element as Element>::ElemOf<P::ChunkItem>> {
        let puller = unsafe { &mut *(&mut self.puller as *mut P) };
        match puller.pull().map(K::destruct_chunk) {
            Some((begin_idx, chunk)) => {
                self.begin_idx = begin_idx;
                self.current_chunk = K::into_seq_chunk_iter(chunk);
                self.next()
            }
            None => None,
        }
    }
}

impl<'c, P, K> Iterator for ChunksIter<'c, P, K>
where
    P: ChunkPuller<K> + 'c,
    K: Enumeration,
{
    type Item = <K::Element as Element>::ElemOf<P::ChunkItem>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = K::seq_chunk_iter_next(self.begin_idx, &mut self.current_chunk);
        match next.is_some() {
            true => next,
            false => self.next_chunk(),
        }
    }
}
