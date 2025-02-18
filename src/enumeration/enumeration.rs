use super::element::Element;
use core::fmt::Debug;

pub(crate) trait EnumerationCore: Send + Sync {
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
    ) -> Option<<Self::ElemKindCore as Element>::IterOf<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync;

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I>;
}

pub trait Enumeration: EnumerationCore<ElemKindCore = Self::Element> {
    type Element: Element;
}
