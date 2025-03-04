use crate::{enumeration::EnumerationCore, Element};

pub struct PulledChunkIter<I, E>
where
    E: EnumerationCore,
    I: Iterator + Default,
    I::Item: Send + Sync,
{
    begin_idx: E::BeginIdx,
    chunk: E::SeqChunkIter<I>,
}

impl<I, E> PulledChunkIter<I, E>
where
    E: EnumerationCore,
    I: Iterator + Default,
    I::Item: Send + Sync,
{
    pub(crate) fn new(begin_idx: E::BeginIdx, chunk: E::SeqChunkIter<I>) -> Self {
        Self { begin_idx, chunk }
    }
}

impl<I, E> Iterator for PulledChunkIter<I, E>
where
    E: EnumerationCore,
    I: Iterator + Default,
    I::Item: Send + Sync,
{
    type Item = <E::ElemKindCore as Element>::ElemOf<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        E::seq_chunk_iter_next(self.begin_idx, &mut self.chunk)
    }
}
