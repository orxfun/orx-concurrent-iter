use super::element::Element;
use crate::pullers::PulledChunkIter;
use core::fmt::Debug;

pub trait EnumerationCore: Send + Sync + 'static {
    type ElemKindCore: Element;

    type BeginIdx: Default + Copy + PartialEq + Debug;

    type SeqChunkIter<I>: Default + Iterator
    where
        I: Iterator + Default,
        I::Item: Send + Sync;

    fn into_begin_idx(begin_idx: usize) -> Self::BeginIdx;

    fn new_element<T>(idx: usize, item: T) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync;

    fn new_chunk<T, I>(begin_idx: usize, chunk: I) -> <Self::ElemKindCore as Element>::IterOf<I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    fn new_pulled_chunk_iter<I>(begin_idx: Self::BeginIdx, chunk: I) -> PulledChunkIter<I, Self>
    where
        Self: Sized,
        I: ExactSizeIterator + Default,
        I::Item: Send + Sync;

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

    fn into_seq_chunk_iter<I>(iter: I) -> Self::SeqChunkIter<I>
    where
        I: Iterator + Default,
        I::Item: Send + Sync;

    fn new_seq_chunk_item<T>(
        begin_idx: Self::BeginIdx,
        within_chunk_idx: usize,
        item: T,
    ) -> <Self::ElemKindCore as Element>::ElemOf<T>
    where
        T: Send + Sync;

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
