use super::ChunkPuller;
use crate::enumeration::{Element, Enumeration, Regular};
use core::marker::PhantomData;

pub struct ChunkIter<'c, P, E = Regular>
where
    P: ChunkPuller<E> + 'c,
    E: Enumeration,
{
    puller: P,
    begin_idx: E::BeginIdx,
    current_chunk: E::SeqChunkIter<P::Iter<'c>>,
    phantom: PhantomData<E>,
}

impl<'c, P, E> ChunkIter<'c, P, E>
where
    P: ChunkPuller<E> + 'c,
    E: Enumeration,
{
    pub(crate) fn new(puller: P) -> Self {
        Self {
            puller,
            begin_idx: Default::default(),
            current_chunk: Default::default(),
            phantom: PhantomData,
        }
    }

    fn next_chunk(&mut self) -> Option<<E::Element as Element>::ElemOf<P::ChunkItem>> {
        let puller = unsafe { &mut *(&mut self.puller as *mut P) };
        match puller.pull().map(E::destruct_chunk) {
            Some((begin_idx, chunk)) => {
                self.begin_idx = begin_idx;
                self.current_chunk = E::into_seq_chunk_iter(chunk);
                self.next()
            }
            None => None,
        }
    }
}

impl<'c, P, E> Iterator for ChunkIter<'c, P, E>
where
    P: ChunkPuller<E> + 'c,
    E: Enumeration,
{
    type Item = <E::Element as Element>::ElemOf<P::ChunkItem>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = E::seq_chunk_iter_next(self.begin_idx, &mut self.current_chunk);
        match next.is_some() {
            true => next,
            false => self.next_chunk(),
        }
    }
}
