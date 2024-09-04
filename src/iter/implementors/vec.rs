use crate::{
    iter::{buffered::vec::BufferedVec, no_leak_iter::NoLeakIter},
    next::NextChunk,
    ConcurrentIter, ConcurrentIterX, Next,
};
use std::{
    mem::{ManuallyDrop, MaybeUninit},
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A concurrent iterator over a vector, consuming the vector and yielding its elements.
pub struct ConIterOfVec<T: Send + Sync> {
    ptr: *mut T,
    vec_len: usize,
    vec_cap: usize,
    counter: AtomicUsize,
}

impl<T: Send + Sync> Drop for ConIterOfVec<T> {
    fn drop(&mut self) {
        // # SAFETY
        // null ptr indicates that the data is already taken out of this iterator
        // by a consuming method such as `into_seq_iter`
        if !self.ptr.is_null() {
            unsafe { self.drop_elements_in_place(self.num_taken()..self.vec_len) };
            let _vec_to_drop = unsafe { Vec::from_raw_parts(self.ptr, 0, self.vec_cap) };
        }
    }
}

impl<T: Send + Sync> std::fmt::Debug for ConIterOfVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::helpers::fmt_iter(f, "ConIterOfVec", Some(self.vec_len), &self.counter)
    }
}

impl<T: Send + Sync> From<Vec<T>> for ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T: Send + Sync> ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    pub fn new(mut vec: Vec<T>) -> Self {
        let (vec_len, vec_cap) = (vec.len(), vec.capacity());
        let ptr = vec.as_mut_ptr();
        let _ = ManuallyDrop::new(vec);
        Self {
            ptr,
            vec_len,
            vec_cap,
            counter: 0.into(),
        }
    }

    pub(crate) fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.vec_len {
            true => Some(begin_idx),
            _ => None,
        }
    }

    fn get(&self, item_idx: usize) -> Option<T> {
        match item_idx < self.vec_len {
            // SAFETY: only one thread can access the `item_idx`-th position and `item_idx` is in bounds
            true => Some(unsafe { self.take_one(item_idx) }),
            _ => None,
        }
    }

    unsafe fn take_one(&self, item_idx: usize) -> T {
        let src_ptr = self.ptr.add(item_idx);

        let mut value = MaybeUninit::<T>::uninit();
        value.as_mut_ptr().swap(src_ptr);

        value.assume_init()
    }

    unsafe fn drop_elements_in_place(&self, range: Range<usize>) {
        for i in range {
            self.ptr.add(i).drop_in_place();
        }
    }

    fn num_taken(&self) -> usize {
        self.counter.load(Ordering::Acquire).min(self.vec_len)
    }

    pub(crate) unsafe fn take_slice(
        &self,
        begin_idx: usize,
        len: usize,
    ) -> impl ExactSizeIterator<Item = T> + '_ {
        let end_idx = (begin_idx + len).min(self.vec_len);
        let iter = (begin_idx..end_idx).map(|i| self.take_one(i));
        NoLeakIter::from(iter)
    }
}

unsafe impl<T: Send + Sync> Sync for ConIterOfVec<T> {}

unsafe impl<T: Send + Sync> Send for ConIterOfVec<T> {}

// AtomicIter -> ConcurrentIter

impl<T: Send + Sync> ConcurrentIterX for ConIterOfVec<T> {
    type Item = T;

    type SeqIter = std::vec::IntoIter<T>;

    type BufferedIterX = BufferedVec<T>;

    fn into_seq_iter(mut self) -> Self::SeqIter {
        let num_taken = self.counter.load(Ordering::Acquire).min(self.vec_len);
        let ptr = self.ptr;

        self.ptr = std::ptr::null_mut(); // to avoid double free on drop

        match num_taken {
            0 => {
                let vec = unsafe { Vec::from_raw_parts(ptr, self.vec_len, self.vec_cap) };
                vec.into_iter()
            }
            _ => {
                let right_len = self.vec_len - num_taken;
                for i in 0..right_len {
                    let dst = unsafe { ptr.add(i) };
                    let src = unsafe { ptr.add(i + num_taken) };
                    unsafe { dst.swap(src) };
                }
                let vec = unsafe { Vec::from_raw_parts(ptr, right_len, self.vec_cap) };
                vec.into_iter()
            }
        }
    }

    fn next_chunk_x(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = Self::Item>> {
        let begin_idx = self
            .progress_and_get_begin_idx(chunk_size)
            .unwrap_or(self.vec_len);
        let end_idx = (begin_idx + chunk_size).min(self.vec_len).max(begin_idx);

        match begin_idx < end_idx {
            true => Some(unsafe { self.take_slice(begin_idx, chunk_size) }),
            false => None,
        }
    }

    fn next(&self) -> Option<Self::Item> {
        let idx = self.counter.fetch_add(1, Ordering::Acquire);
        self.get(idx)
    }

    fn skip_to_end(&self) {
        let num_taken_before = self.counter.fetch_max(self.vec_len, Ordering::Acquire);
        if num_taken_before < self.vec_len {
            unsafe { self.drop_elements_in_place(num_taken_before..self.vec_len) };
        }
    }

    fn try_get_len(&self) -> Option<usize> {
        let current = self.counter.load(Ordering::Acquire);
        let initial_len = self.vec_len;
        let len = match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        };
        Some(len)
    }

    #[inline(always)]
    fn try_get_initial_len(&self) -> Option<usize> {
        Some(self.vec_len)
    }
}

impl<T: Send + Sync> ConcurrentIter for ConIterOfVec<T> {
    type BufferedIter = Self::BufferedIterX;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        let idx = self.counter.fetch_add(1, Ordering::Acquire);
        self.get(idx).map(|value| Next { idx, value })
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(chunk_size)
            .unwrap_or(self.vec_len);
        let end_idx = (begin_idx + chunk_size).min(self.vec_len).max(begin_idx);

        match begin_idx < end_idx {
            true => {
                let values = unsafe { self.take_slice(begin_idx, chunk_size) };
                Some(NextChunk { begin_idx, values })
            }
            false => None,
        }
    }
}
