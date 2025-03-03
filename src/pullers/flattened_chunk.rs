use crate::{Element, Enumeration};

pub struct FlattenedChunk<I, E>
where
    E: Enumeration,
    I: Iterator,
    I::Item: Send + Sync,
{
    begin_idx: E::BeginIdx,
    chunk: I,
}

impl<I, E> FlattenedChunk<I, E>
where
    E: Enumeration,
    I: Iterator,
    I::Item: Send + Sync,
{
    pub(crate) fn new(begin_idx: E::BeginIdx, chunk: I) -> Self {
        Self { begin_idx, chunk }
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
        // self.chunk
        //     .next()
        //     .map(|value| E::new_element_using_idx(self.begin_idx, value))
        None
    }
}
