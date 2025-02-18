use core::{fmt::Debug, iter::Enumerate};

pub(crate) trait NextKindCore {
    type NextChunk<T, I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    type BeginIdx: Default + Copy + PartialEq + Debug;

    type SeqChunkIter<I>: Default
    where
        I: Iterator + Default;

    fn destruct_chunk<T, I>(chunk: Self::NextChunk<T, I>) -> (Self::BeginIdx, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I>;
}

pub trait NextKind: NextKindCore {
    type Next<T>: Send + Sync
    where
        T: Send + Sync;

    // TODO: to be removed from public api
    fn seq_chunk_iter_next<I>(
        begin_idx: Self::BeginIdx,
        seq_iter: &mut Self::SeqChunkIter<I>,
    ) -> Option<Self::Next<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync;
}

pub struct Regular;
impl NextKindCore for Regular {
    type NextChunk<T, I>
        = I
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    type BeginIdx = ();

    type SeqChunkIter<I>
        = I
    where
        I: Iterator + Default;

    fn destruct_chunk<T, I>(chunk: Self::NextChunk<T, I>) -> (Self::BeginIdx, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        ((), chunk)
    }

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I> {
        iter
    }
}
impl NextKind for Regular {
    type Next<T: Send + Sync> = T;

    fn seq_chunk_iter_next<I>(
        _: Self::BeginIdx,
        seq_iter: &mut Self::SeqChunkIter<I>,
    ) -> Option<Self::Next<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync,
    {
        seq_iter.next()
    }
}

pub struct Enumerated;
impl NextKindCore for Enumerated {
    type NextChunk<T, I>
        = (usize, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;

    type BeginIdx = usize;

    type SeqChunkIter<I>
        = Enumerate<I>
    where
        I: Iterator + Default;

    fn destruct_chunk<T, I>(chunk: Self::NextChunk<T, I>) -> (Self::BeginIdx, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>,
    {
        chunk
    }

    fn into_seq_chunk_iter<I: Iterator + Default>(iter: I) -> Self::SeqChunkIter<I> {
        iter.enumerate()
    }
}
impl NextKind for Enumerated {
    type Next<T: Send + Sync> = (usize, T);

    fn seq_chunk_iter_next<I>(
        begin_idx: Self::BeginIdx,
        seq_iter: &mut Self::SeqChunkIter<I>,
    ) -> Option<Self::Next<I::Item>>
    where
        I: Iterator + Default,
        I::Item: Send + Sync,
    {
        seq_iter.next().map(|(i, x)| (begin_idx + i, x))
    }
}
