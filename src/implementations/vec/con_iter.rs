use super::{chunk_puller::ChunkPullerVec, vec_into_seq_iter::VecIntoSeqIter};
use crate::{
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::ptr_utils::take,
};
use alloc::vec::Vec;
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Concurrent iterator of a [`Vec`].
///
/// It can be created by calling [`into_con_iter`] on a vector.
///
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec![1, 2];
/// let con_iter = vec.into_con_iter();
/// assert_eq!(con_iter.next(), Some(1));
/// assert_eq!(con_iter.next(), Some(2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct ConIterVec<T>
where
    T: Send + Sync,
{
    ptr: *const T,
    vec_len: usize,
    vec_cap: usize,
    counter: AtomicUsize,
}

unsafe impl<T: Send + Sync> Sync for ConIterVec<T> {}

unsafe impl<T: Send + Sync> Send for ConIterVec<T> {}

impl<T> Default for ConIterVec<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T> Drop for ConIterVec<T>
where
    T: Send + Sync,
{
    fn drop(&mut self) {
        let _iter = self.remaining_into_seq_iter();
    }
}

impl<T> ConIterVec<T>
where
    T: Send + Sync,
{
    pub(super) fn new(vec: Vec<T>) -> Self {
        let (vec_len, vec_cap, ptr) = (vec.len(), vec.capacity(), vec.as_ptr());
        let _ = ManuallyDrop::new(vec);
        Self {
            ptr,
            vec_len,
            vec_cap,
            counter: 0.into(),
        }
    }

    pub(super) fn initial_len(&self) -> usize {
        self.vec_len
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
                let first = unsafe { self.ptr.add(begin_idx) }; // ptr + begin_idx is in bounds
                let last = unsafe { self.ptr.add(end_idx - 1) }; // ptr + end_idx - 1 is in bounds
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

impl<T> ConcurrentIter for ConIterVec<T>
where
    T: Send + Sync,
{
    type Item = T;

    type SequentialIter = VecIntoSeqIter<T>;

    type ChunkPuller<'i>
        = ChunkPullerVec<'i, T>
    where
        Self: 'i;

    fn into_seq_iter(mut self) -> Self::SequentialIter {
        self.remaining_into_seq_iter()
    }

    fn skip_to_end(&self) {
        let current = self.counter.fetch_max(self.vec_len, Ordering::Acquire);
        let num_taken_before = current.min(self.vec_len);
        let _iter = self.slice_into_seq_iter(num_taken_before, false);
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1) // ptr + idx is in-bounds
            .map(|idx| unsafe { take(self.ptr.add(idx) as *mut T) })
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.progress_and_get_begin_idx(1) // ptr + idx is in-bounds
            .map(|idx| (idx, unsafe { take(self.ptr.add(idx) as *mut T) }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_taken = self.counter.load(Ordering::Acquire);
        let remaining = self.vec_len.saturating_sub(num_taken);
        (remaining, Some(remaining))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterVec<T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.vec_len.saturating_sub(num_taken)
    }
}
