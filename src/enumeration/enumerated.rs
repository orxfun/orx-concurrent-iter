use super::{
    element::{Element, IdxValue},
    enumeration::{Enumeration, EnumerationCore},
    is_enumerated::IsEnumerated,
};

pub struct Enumerated;

impl IsEnumerated for Enumerated {}

impl EnumerationCore for Enumerated {
    type ElemKindCore = IdxValue;

    type BeginIdx = usize;

    type SeqChunkIter<I>
        = core::iter::Enumerate<I>
    where
        I: Iterator + Default;

    fn new_element<T>(idx: usize, item: T) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync,
    {
        (idx, item)
    }

    fn new_chunk<T, I>(begin_idx: usize, chunk: I) -> <Self::ElemKindCore as Element>::IterOf<I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        (begin_idx, chunk)
    }

    fn destruct_chunk<T, I>(
        chunk: <Self::ElemKindCore as Element>::IterOf<I>,
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

    fn new_seq_chunk_item<T>(
        begin_idx: Self::BeginIdx,
        within_chunk_idx: Self::BeginIdx,
        item: T,
    ) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync,
    {
        (begin_idx + within_chunk_idx, item)
    }

    // test

    #[cfg(test)]
    fn new_element_using_idx<T>(
        idx: Self::BeginIdx,
        item: T,
    ) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync,
    {
        (idx, item)
    }
}

impl Enumeration for Enumerated {
    type Element = IdxValue;
}
