use super::{chunks_iter_vec::ChunksIterVec, vec_into_seq_iter::VecIntoSeqIter};
use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
    implementations::ptr_utils::take,
};
use alloc::vec::Vec;
use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterVec<T, E = Regular>
where
    T: Send + Sync,
    E: Enumeration,
{
    ptr: *const T,
    vec_len: usize,
    vec_cap: usize,
    counter: AtomicUsize,
    phantom: PhantomData<E>,
}

unsafe impl<T: Send + Sync, E: Enumeration> Sync for ConIterVec<T, E> {}

unsafe impl<T: Send + Sync, E: Enumeration> Send for ConIterVec<T, E> {}

impl<T, E> Default for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T, E> Drop for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn drop(&mut self) {
        let _iter = self.remaining_into_seq_iter();
    }
}

impl<T, E> ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(crate) fn new(vec: Vec<T>) -> Self {
        let (vec_len, vec_cap, ptr) = (vec.len(), vec.capacity(), vec.as_ptr());
        let _ = ManuallyDrop::new(vec);
        Self {
            ptr,
            vec_len,
            vec_cap,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    fn transform<E2: Enumeration>(mut self) -> ConIterVec<T, E2> {
        let (ptr, counter) = (self.ptr, self.counter.load(Ordering::Acquire).into());
        self.ptr = core::ptr::null();
        ConIterVec {
            ptr,
            vec_len: self.vec_len,
            vec_cap: self.vec_cap,
            counter,
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

    pub(super) fn progress_and_get_chunk_pointers(
        &self,
        chunk_size: usize,
    ) -> Option<(usize, *const T, *const T)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size).min(self.vec_len).max(begin_idx);
                let first = unsafe { self.ptr.add(begin_idx) } as *const T; // ptr + begin_idx is in bounds
                let last = unsafe { self.ptr.add(end_idx - 1) } as *const T; // ptr + end_idx - 1 is in bounds
                (begin_idx, first, last)
            })
    }

    fn remaining_into_seq_iter(&mut self) -> VecIntoSeqIter<T> {
        // # SAFETY
        // null ptr indicates that the data is already taken out of this iterator
        // by a consuming method such as `into_seq_iter`
        match self.ptr.is_null() {
            true => Default::default(),
            false => {
                let num_taken = self.counter.load(Ordering::Acquire).min(self.vec_len);
                let iter = self.slice_into_seq_iter(num_taken, true);
                self.ptr = core::ptr::null();
                iter
            }
        }
    }

    fn slice_into_seq_iter(&self, num_taken: usize, drop_vec: bool) -> VecIntoSeqIter<T> {
        let p = self.ptr;
        let completed = num_taken == self.vec_len;

        let (first, last, current) = match completed {
            true => (p, p, p),
            false => {
                let first = p;
                let last = unsafe { first.add(self.vec_len - 1) }; // self.vec_len is positive here
                let current = unsafe { first.add(num_taken) }; // first + num_taken is in bounds
                (first, last, current)
            }
        };

        let drop_vec_capacity = drop_vec.then_some(self.vec_cap);
        VecIntoSeqIter::new(completed, first, last, current, drop_vec_capacity)
    }
}

impl<T, E> ConcurrentIter<E> for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = T;

    type SeqIter = VecIntoSeqIter<T>;

    type ChunkPuller<'i>
        = ChunksIterVec<'i, T, E>
    where
        Self: 'i;

    type Regular = ConIterVec<T, Regular>;

    type Enumerated = ConIterVec<T, Enumerated>;

    fn into_seq_iter(mut self) -> Self::SeqIter {
        self.remaining_into_seq_iter()
    }

    fn enumerated(self) -> Self::Enumerated
    where
        E: IsNotEnumerated,
    {
        self.transform()
    }

    fn not_enumerated(self) -> Self::Regular
    where
        E: IsEnumerated,
    {
        self.transform()
    }

    fn skip_to_end(&self) {
        let current = self.counter.fetch_max(self.vec_len, Ordering::Acquire);
        let num_taken_before = current.min(self.vec_len);
        let _iter = self.slice_into_seq_iter(num_taken_before, false);
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        self.progress_and_get_begin_idx(1) // ptr + idx is in-bounds
            .map(|idx| E::new_element(idx, unsafe { take(self.ptr.add(idx) as *mut T) }))
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}
