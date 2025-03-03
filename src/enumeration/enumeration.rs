use super::element::Element;
use core::fmt::Debug;

pub trait EnumerationCore: Send + Sync + 'static {
    type ElemKindCore: Element;

    type BeginIdx: Default + Copy + PartialEq + Debug;

    type SeqChunkIter<I>: Default
    where
        I: Iterator + Default;

    fn new_element<T>(idx: usize, item: T) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync;

    fn new_chunk<T, I>(begin_idx: usize, chunk: I) -> <Self::ElemKindCore as Element>::IterOf<I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    fn destruct_chunk<T, I>(
        chunk: <Self::ElemKindCore as Element>::IterOf<I>,
    ) -> (Self::BeginIdx, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    fn seq_chunk_iter_next<I>(
        begin_idx: Self::BeginIdx,
        seq_iter: &mut Self::SeqChunkIter<I>,
    ) -> Option<<Self::ElemKindCore as Element>::ElemOf<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync;

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I>;

    // test

    #[cfg(test)]
    fn new_element_using_idx<T>(
        idx: Self::BeginIdx,
        item: T,
    ) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync;
}

pub trait Enumeration: EnumerationCore<ElemKindCore = Self::Element> {
    type Element: Element;
}
