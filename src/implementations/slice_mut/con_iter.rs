// use super::chunk_puller::ChunkPullerSlice;
use crate::{
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::slice_mut::chunk_puller::ChunkPullerSliceMut,
};
use core::{
    iter::Skip,
    sync::atomic::{AtomicUsize, Ordering},
};

// TODO: documentation update

/// Concurrent iterator of a mutable slice.
///
/// It can be created by calling [`into_con_iter`] on a mutable slice.
///
/// Alternatively, it can be created calling [`con_iter`] on the type
/// that owns the slice.
///
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
/// [`con_iter`]: crate::ConcurrentIterable::con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// // &[T]: IntoConcurrentIter
/// let vec = vec![0, 1, 2, 3];
/// let slice = &vec[1..3];
/// let con_iter = slice.into_con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
///
/// // Vec<T>: ConcurrentIterable
/// let vec = vec![1, 2];
/// let con_iter = vec.con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct ConIterSliceMut<'a, T> {
    slice: &'a mut [T],
    p: *mut T,
    counter: AtomicUsize,
}

unsafe impl<'a, T> Sync for ConIterSliceMut<'a, T> {}

unsafe impl<'a, T> Send for ConIterSliceMut<'a, T> {}

impl<T> Default for ConIterSliceMut<'_, T> {
    fn default() -> Self {
        Self::new(&mut [])
    }
}

impl<'a, T> ConIterSliceMut<'a, T> {
    pub(crate) fn new(slice: &'a mut [T]) -> Self {
        Self {
            p: slice.as_mut_ptr(),
            slice,
            counter: 0.into(),
        }
    }

    pub(super) fn slice(&self) -> &&'a mut [T] {
        &self.slice
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.slice.len() {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) unsafe fn progress_and_get_slice(
        &self,
        chunk_size: usize,
    ) -> Option<(usize, &'a mut [T])> {
        let slice_len = self.slice.len();

        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size).min(slice_len).max(begin_idx);

                let ptr = unsafe { self.p.add(begin_idx) };
                let len = end_idx - begin_idx;
                let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };

                (begin_idx, slice)
            });
        None
    }
}

impl<'a, T> ConcurrentIter for ConIterSliceMut<'a, T> {
    type Item = &'a mut T;

    type SequentialIter = Skip<core::slice::IterMut<'a, T>>;

    type ChunkPuller<'i>
        = ChunkPullerSliceMut<'i, 'a, T>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter_mut().skip(current)
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }

    fn next(&self) -> Option<Self::Item> {
        self.progress_and_get_begin_idx(1).map(|idx| {
            let ptr = unsafe { self.p.add(idx) };
            unsafe { &mut *ptr }
        })
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.progress_and_get_begin_idx(1).map(|idx| {
            let ptr = unsafe { self.p.add(idx) };
            (idx, unsafe { &mut *ptr })
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_taken = self.counter.load(Ordering::Acquire);
        let remaining = self.slice.len().saturating_sub(num_taken);
        (remaining, Some(remaining))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterSliceMut<'_, T> {
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.slice.len().saturating_sub(num_taken)
    }
}
