use super::chunks_iter_slice_ref::ChunksIterSliceRef;
use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
};
use core::{
    iter::Skip,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterSliceRef<'a, T, E = Regular>
where
    T: Send + Sync,
    E: Enumeration,
{
    slice: &'a [T],
    counter: AtomicUsize,
    phantom: PhantomData<E>,
}

impl<'a, T, K> Default for ConIterSliceRef<'a, T, K>
where
    T: Send + Sync,
    K: Enumeration,
{
    fn default() -> Self {
        Self {
            slice: &[],
            counter: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, E> ConIterSliceRef<'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(crate) fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.slice.len() {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) fn progress_and_get_chunk(&self, chunk_size: usize) -> Option<(usize, &'a [T])> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.slice.len())
                    .max(begin_idx);
                (begin_idx, &self.slice[begin_idx..end_idx])
            })
    }
}

impl<'a, T, E> ConcurrentIter<E> for ConIterSliceRef<'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = &'a T;

    type SeqIter = Skip<core::slice::Iter<'a, T>>;

    type Regular = ConIterSliceRef<'a, T, Regular>;

    type Enumerated = ConIterSliceRef<'a, T, Enumerated>;

    type ChunkPuller<'i>
        = ChunksIterSliceRef<'i, 'a, T, E>
    where
        Self: 'i;

    // into

    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
    }

    fn enumerated(self) -> Self::Enumerated
    where
        E: IsNotEnumerated,
    {
        ConIterSliceRef {
            slice: self.slice,
            counter: self.counter,
            phantom: PhantomData,
        }
    }

    fn not_enumerated(self) -> Self::Regular
    where
        E: IsEnumerated,
    {
        ConIterSliceRef {
            slice: self.slice,
            counter: self.counter,
            phantom: PhantomData,
        }
    }

    // iter

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        self.progress_and_get_begin_idx(1)
            .map(|idx| E::new_element(idx, &self.slice[idx]))
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }

    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<
        <<E as Enumeration>::Element as Element>::IterOf<
            <Self::ChunkPuller<'_> as crate::chunk_puller::ChunkPuller<E>>::Iter,
        >,
    > {
        self.chunks_iter(chunk_size).next()
    }
}
