use crate::{Element, Enumeration};

pub struct PulledChunkIter<I, E>
where
    E: Enumeration,
    I: Iterator + Default,
    I::Item: Send + Sync,
{
    begin_idx: E::BeginIdx,
    chunk: E::SeqChunkIter<I>,
}

impl<I, E> Iterator for PulledChunkIter<I, E>
where
    E: Enumeration,
    I: Iterator + Default,
    I::Item: Send + Sync,
{
    type Item = <E::Element as Element>::ElemOf<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        E::seq_chunk_iter_next(self.begin_idx, &mut self.chunk)
    }
}
