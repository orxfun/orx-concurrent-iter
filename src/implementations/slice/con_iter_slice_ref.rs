use super::chunks_iter_slice_ref::ChunksIterSliceRef;
use crate::{
    concurrent_iter::ConcurrentIter,
    next::{NextKind, Regular},
    Enumerated,
};
use core::{
    iter::Skip,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterSliceRef<'a, T, K = Regular>
where
    K: NextKind,
{
    slice: &'a [T],
    counter: AtomicUsize,
    phantom: PhantomData<K>,
}

impl<'a, T, K> Default for ConIterSliceRef<'a, T, K>
where
    K: NextKind,
{
    fn default() -> Self {
        Self {
            slice: &[],
            counter: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, K> ConIterSliceRef<'a, T, K>
where
    K: NextKind,
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

impl<'a, T, K> ConcurrentIter<K> for ConIterSliceRef<'a, T, K>
where
    K: NextKind,
{
    type Item = &'a T;

    type SeqIter = Skip<core::slice::Iter<'a, T>>;

    type Regular = ConIterSliceRef<'a, T, Regular>;

    type Enumerated = ConIterSliceRef<'a, T, Enumerated>;

    type ChunkPuller<'i>
        = ChunksIterSliceRef<'i, 'a, T, K>
    where
        Self: 'i;

    // into

    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
    }

    // enumeration

    fn as_enumerated(&self) -> Self::Enumerated {
        let current = self.counter.load(Ordering::Acquire);
        ConIterSliceRef {
            slice: self.slice,
            counter: current.into(),
            phantom: PhantomData,
        }
    }

    // iter

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }

    fn next(&self) -> Option<K::Next<Self::Item>> {
        self.progress_and_get_begin_idx(1)
            .map(|begin_idx| K::new_next(begin_idx, &self.slice[begin_idx]))
    }

    fn in_chunks(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}
