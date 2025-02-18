use super::{
    element::{Element, IdxValue},
    enumeration::{Enumeration, EnumerationCore},
};

pub struct Enumerated;

impl EnumerationCore for Enumerated {
    type ElemKindCore = IdxValue;

    type BeginIdx = usize;

    type SeqChunkIter<I>
        = core::iter::Enumerate<I>
    where
        I: Iterator + Default;

    fn new_elem<T>(idx: usize, item: T) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync,
    {
        (idx, item)
    }

    fn new_chunk<T, I>(begin_idx: usize, chunk: I) -> <Self::ElemKindCore as Element>::ElemOf<I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        (begin_idx, chunk)
    }

    fn destruct_chunk<T, I>(
        chunk: <Self::ElemKindCore as Element>::ElemOf<I>,
    ) -> (Self::BeginIdx, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        chunk
    }

    fn seq_chunk_iter_next<I>(
        begin_idx: Self::BeginIdx,
        seq_iter: &mut Self::SeqChunkIter<I>,
    ) -> Option<<Self::ElemKindCore as Element>::ElemOf<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync,
    {
        seq_iter.next().map(|(i, x)| (begin_idx + i, x))
    }

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I> {
        iter.enumerate()
    }
}

impl Enumeration for Enumerated {
    type Element = IdxValue;
}
