use crate::{
    chunk_puller::DoNothingChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
};
use core::{
    marker::PhantomData,
    ops::{Add, Range},
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterRange<T, E = Regular>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + Iterator<Item = T>,
{
    begin: usize,
    len: usize,
    counter: AtomicUsize,
    phantom: PhantomData<(T, E)>,
}

impl<T, E> Default for ConIterRange<T, E>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + Iterator<Item = T>,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T, E> ConIterRange<T, E>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + Iterator<Item = T>,
{
    pub(crate) fn new(range: Range<T>) -> Self {
        let begin: usize = range.start.into();
        let end: usize = range.end.into();
        let len = end - begin;
        Self {
            begin,
            len,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.len {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) fn progress_and_get_range(&self, chunk_size: usize) -> Option<(usize, T, T)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size).min(self.len).max(begin_idx);
                let begin = self.begin + begin_idx;
                let end = self.begin + end_idx;
                (begin_idx, begin.into(), end.into())
            })
    }
}

impl<T, E> ConcurrentIter<E> for ConIterRange<T, E>
where
    T: Send + Sync + Copy + From<usize> + Into<usize> + Add<T, Output = T>,
    E: Enumeration,
    Range<T>: Default + Iterator<Item = T>,
{
    type Item = T;

    type SeqIter = Range<T>;

    type ChunkPuller<'i>
        = DoNothingChunkPuller<E, T>
    where
        Self: 'i;

    type Regular = ConIterRange<T, Regular>;

    type Enumerated = ConIterRange<T, Enumerated>;

    fn into_seq_iter(self) -> Self::SeqIter {
        todo!()
    }

    fn enumerated(self) -> Self::Enumerated
    where
        E: IsNotEnumerated,
    {
        todo!()
    }

    fn not_enumerated(self) -> Self::Regular
    where
        E: IsEnumerated,
    {
        todo!()
    }

    fn skip_to_end(&self) {
        todo!()
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        todo!()
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        todo!()
    }
}
