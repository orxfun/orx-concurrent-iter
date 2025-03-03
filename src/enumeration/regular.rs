use super::{
    element::{Element, Value},
    enumeration::{Enumeration, EnumerationCore},
    is_enumerated::IsNotEnumerated,
};

pub struct Regular;

impl IsNotEnumerated for Regular {}

impl EnumerationCore for Regular {
    type ElemKindCore = Value;

    type BeginIdx = ();

    type SeqChunkIter<I>
        = I
    where
        I: Iterator + Default;

    fn new_element<T>(_: usize, item: T) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync,
    {
        item
    }

    fn new_chunk<T, I>(_: usize, chunk: I) -> <Self::ElemKindCore as Element>::IterOf<I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        chunk
    }

    fn destruct_chunk<T, I>(
        chunk: <Self::ElemKindCore as Element>::IterOf<I>,
    ) -> (Self::BeginIdx, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        ((), chunk)
    }

    fn seq_chunk_iter_next<I>(
        _: Self::BeginIdx,
        seq_iter: &mut Self::SeqChunkIter<I>,
    ) -> Option<<Self::ElemKindCore as Element>::ElemOf<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync,
    {
        seq_iter.next()
    }

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I> {
        iter
    }

    // test

    #[cfg(test)]
    fn new_element_using_idx<T>(
        _: Self::BeginIdx,
        item: T,
    ) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync,
    {
        item
    }
}

impl Enumeration for Regular {
    type Element = Value;
}
