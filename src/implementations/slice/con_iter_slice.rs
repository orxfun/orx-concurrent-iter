use super::chunk_puller_slice::ChunkPullerSlice;
use crate::{
    concurrent_iter::{ConcurrentIter, ConcurrentIterEnum},
    enumeration::{Element, Enumeration, Regular},
};
use core::{
    iter::Skip,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterSlice<'a, T, E = Regular>
where
    T: Send + Sync,
    E: Enumeration,
{
    slice: &'a [T],
    counter: AtomicUsize,
    phantom: PhantomData<E>,
}

impl<'a, T, E> Default for ConIterSlice<'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn default() -> Self {
        Self {
            slice: &[],
            counter: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, E> ConIterSlice<'a, T, E>
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

    pub(super) fn progress_and_get_slice(&self, chunk_size: usize) -> Option<(usize, &'a [T])> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.slice.len())
                    .max(begin_idx);
                (begin_idx, &self.slice[begin_idx..end_idx])
            })
    }
}

impl<'a, T, E> ConcurrentIterEnum<E, &'a T> for ConIterSlice<'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type EnumerationOf<E2>
        = ConIterSlice<'a, T, E2>
    where
        E2: Enumeration;

    fn into_enumeration_of<E2: Enumeration>(self) -> Self::EnumerationOf<E2> {
        let counter = self.counter.load(Ordering::Acquire).into();
        ConIterSlice {
            slice: self.slice,
            counter,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, E> ConcurrentIter<E> for ConIterSlice<'a, T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = &'a T;

    type SeqIter = Skip<core::slice::Iter<'a, T>>;

    type ChunkPuller<'i>
        = ChunkPullerSlice<'i, 'a, T, E>
    where
        Self: 'i;

    // into

    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
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
}
