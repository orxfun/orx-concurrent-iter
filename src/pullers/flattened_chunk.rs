use crate::{Element, Enumeration};

pub struct FlattenedChunk<I, E>
where
    E: Enumeration,
    I: Iterator,
    I::Item: Send + Sync,
{
    begin_idx: E::BeginIdx,
    chunk: core::iter::Enumerate<I>,
}

impl<I, E> FlattenedChunk<I, E>
where
    E: Enumeration,
    I: Iterator,
    I::Item: Send + Sync,
{
    pub(crate) fn new(begin_idx: E::BeginIdx, chunk: I) -> Self {
        Self {
            begin_idx,
            chunk: chunk.enumerate(),
        }
    }
}

impl<I, E> Iterator for FlattenedChunk<I, E>
where
    E: Enumeration,
    I: Iterator,
    I::Item: Send + Sync,
{
    type Item = <E::Element as Element>::ElemOf<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunk
            .next()
            .map(|(i, value)| E::new_seq_chunk_item(self.begin_idx, i, value))
    }
}
