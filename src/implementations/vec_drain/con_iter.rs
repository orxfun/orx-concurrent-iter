use crate::{
    concurrent_iter::ConcurrentIter,
    exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::{
        array_utils::{ArrayChunkPuller, ArrayConIter, ArrayIntoSeqIter, ChunkPointers},
        ptr_utils::take,
    },
};
use alloc::vec::Vec;
use core::{
    ops::{Bound, Range, RangeBounds},
    sync::atomic::{AtomicUsize, Ordering},
};

/// Concurrent drain iterator of a [`Vec`]:
///
/// * once the iterator is dropped, the source vector will not be de-allocated;
///   however, it will be empty without any elements;
/// * none, some or all elements may be traversed and returned by the concurrent iterator;
///   regardless, all elements will be cleaned up.
///
/// TODO: It can be created by calling [`?`] on a vector.
///
/// # Examples
///
/// TODO: update the example for draining iterator.
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
pub struct ConIterVecDrain<'a, T>
where
    T: Send + Sync,
{
    vec: &'a mut Vec<T>,
    range: Range<usize>,
    vec_len: usize,
    ptr: *const T,
    counter: AtomicUsize,
}

unsafe impl<T: Send + Sync> Sync for ConIterVecDrain<'_, T> {}

unsafe impl<T: Send + Sync> Send for ConIterVecDrain<'_, T> {}

impl<T> Drop for ConIterVecDrain<'_, T>
where
    T: Send + Sync,
{
    fn drop(&mut self) {
        let iter = self.remaining_into_seq_iter();
        drop(iter);

        let right_start = self.range.end;
        let right_len = self.vec_len - self.range.end;
        let left_len = self.range.start;
        let new_len = left_len + right_len;

        let ptr = self.vec.as_ptr();
        if right_len > 0 {
            let src = unsafe { ptr.add(right_start) };
            let dst = unsafe { ptr.add(left_len) as *mut T };
            unsafe { core::ptr::copy(src, dst, right_len) };
        }

        unsafe { self.vec.set_len(new_len) };
    }
}

impl<'a, T> ConIterVecDrain<'a, T>
where
    T: Send + Sync,
{
    /// Creates a new concurrent draining iterator over the `vec` for the given `range`.
    ///
    /// # Panics
    ///
    /// Panics:
    ///
    /// * if the starting point of the `range` is greater than the ending point; or
    /// * if the ending point of the `range` is greater than `vec.len()`.
    pub(super) fn new<R>(vec: &'a mut Vec<T>, range: R) -> Self
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Excluded(x) => x + 1,
            Bound::Included(x) => *x,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Excluded(x) => *x,
            Bound::Included(x) => x + 1,
            Bound::Unbounded => vec.len(),
        };
        let range = start..end;

        assert!(range.start <= range.end);
        assert!(range.end <= vec.len());

        let vec_len = vec.len();
        let ptr = vec.as_ptr();
        let counter = range.start.into();

        // TODO: write safety note here
        unsafe { vec.set_len(range.start) };

        Self {
            vec,
            range,
            vec_len,
            ptr,
            counter,
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.range.end {
            true => Some(begin_idx),
            _ => None,
        }
    }

    fn remaining_into_seq_iter(&mut self) -> ArrayIntoSeqIter<T> {
        // # SAFETY
        // null ptr indicates that the data is already taken out of this iterator
        // by a consuming method such as `into_seq_iter`
        match self.ptr.is_null() {
            true => Default::default(),
            false => {
                let num_taken = self.counter.load(Ordering::Acquire).min(self.range.end);
                let iter = self.slice_into_seq_iter(num_taken);
                self.ptr = core::ptr::null();
                iter
            }
        }
    }

    fn slice_into_seq_iter(&self, num_taken: usize) -> ArrayIntoSeqIter<T> {
        let completed = num_taken == self.range.end;
        let (last, current) = match completed {
            true => (core::ptr::null(), core::ptr::null()),
            false => {
                // SAFETY: self.range.end is positive here, would be completed o/w
                let last = unsafe { self.ptr.add(self.range.end - 1) };
                // SAFETY: first + num_taken is in bounds
                let current = unsafe { self.ptr.add(num_taken) };
                (last, current)
            }
        };

        ArrayIntoSeqIter::new(current, last, None)
    }
}

impl<T> ArrayConIter for ConIterVecDrain<'_, T>
where
    T: Send + Sync,
{
    type Item = T;

    fn progress_and_get_chunk_pointers2(
        &self,
        chunk_size: usize,
    ) -> Option<ChunkPointers<Self::Item>> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size).min(self.range.end).max(begin_idx);
                let first = unsafe { self.ptr.add(begin_idx) }; // ptr + begin_idx is in bounds
                let last = unsafe { self.ptr.add(end_idx - 1) }; // ptr + end_idx - 1 is in bounds
                ChunkPointers {
                    begin_idx,
                    first,
                    last,
                }
            })
    }
}

impl<T> ConcurrentIter for ConIterVecDrain<'_, T>
where
    T: Send + Sync,
{
    type Item = T;

    type SequentialIter = ArrayIntoSeqIter<T>;

    type ChunkPuller<'i>
        = ArrayChunkPuller<'i, Self>
    where
        Self: 'i;

    fn into_seq_iter(mut self) -> Self::SequentialIter {
        self.remaining_into_seq_iter()
    }

    fn skip_to_end(&self) {
        let current = self.counter.fetch_max(self.range.end, Ordering::Acquire);
        let num_taken_before = current.min(self.range.end);
        let _iter = self.slice_into_seq_iter(num_taken_before);
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
        let remaining = self.range.end.saturating_sub(num_taken);
        (remaining, Some(remaining))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterVecDrain<'_, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.range.end.saturating_sub(num_taken)
    }
}

#[cfg(test)]
mod tst {
    use super::*;
    use crate::*;
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    #[test]
    fn abc() {
        let n = 4;
        let range = ..;

        let mut vec: Vec<_> = (0..n).map(|x| x.to_string()).collect();

        {
            let iter = ConIterVecDrain::new(&mut vec, range);
            // let iter = vec.drain(range);
            let bx = Box::new(iter);
            Box::leak(bx);
            // while let Some(x) = iter.next() {
            //     dbg!(x);
            // }
        }

        dbg!(&vec);

        // assert_eq!(vec.len(), 33);
    }
}
