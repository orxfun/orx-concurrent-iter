use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
};
use alloc::vec::Vec;
use core::{
    iter::Skip,
    marker::PhantomData,
    mem::ManuallyDrop,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::chunks_iter_vec::ChunksIterVec;

pub struct ConIterVec<T, E = Regular>
where
    T: Send + Sync,
    E: Enumeration,
{
    ptr: *mut T,
    vec_len: usize,
    vec_cap: usize,
    counter: AtomicUsize,
    phantom: PhantomData<E>,
}

impl<T, E> Default for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T, E> ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(crate) fn new(mut vec: Vec<T>) -> Self {
        let (vec_len, vec_cap) = (vec.len(), vec.capacity());
        let ptr = vec.as_mut_ptr();
        let _ = ManuallyDrop::new(vec);
        Self {
            ptr,
            vec_len,
            vec_cap,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.vec_len {
            true => Some(begin_idx),
            _ => None,
        }
    }

    // pub(super) fn progress_and_get_chunk(&self, chunk_size: usize) -> Option<(usize, &'a [T])> {
    //     self.progress_and_get_begin_idx(chunk_size)
    //         .map(|begin_idx| {
    //             let end_idx = (begin_idx + chunk_size)
    //                 .min(self.slice.len())
    //                 .max(begin_idx);
    //             (begin_idx, &self.slice[begin_idx..end_idx])
    //         })
    // }
}

impl<T, E> ConcurrentIter<E> for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = T;

    type SeqIter = Skip<alloc::vec::IntoIter<T>>;

    type ChunkPuller<'i>
        = ChunksIterVec<'i, T, E>
    where
        Self: 'i;

    type Regular = ConIterVec<T, Regular>;

    type Enumerated = ConIterVec<T, Enumerated>;

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
