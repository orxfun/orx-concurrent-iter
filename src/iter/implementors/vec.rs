use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        buffered::{buffered_chunk::BufferedChunk, buffered_iter::BufferedIter, vec::BufferedVec},
    },
    next::NextChunk,
    AtomicCounter, ConcurrentIter, Next,
};
use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    mem::{ManuallyDrop, MaybeUninit},
};

/// A concurrent iterator over a vector, consuming the vector and yielding its elements.
#[derive(Debug)]
pub struct ConIterOfVec<T: Send + Sync> {
    vec: UnsafeCell<ManuallyDrop<Vec<T>>>,
    vec_len: usize,
    counter: AtomicCounter,
}

impl<T: Send + Sync> Drop for ConIterOfVec<T> {
    fn drop(&mut self) {
        let current = self.counter().current();
        match current.cmp(&self.vec_len) {
            Ordering::Less => {
                let _remaining_vec_to_be_dropped = unsafe { self.split_off_right(current) };
            }
            _ => {}
        }
    }
}

impl<T: Send + Sync> ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            vec_len: vec.len(),
            vec: ManuallyDrop::new(vec).into(),
            counter: AtomicCounter::new(),
        }
    }

    unsafe fn take_one(&self, item_idx: usize) -> T {
        let vec = &mut *self.vec.get();
        let src_ptr = vec.as_mut_ptr().add(item_idx);

        let mut value = MaybeUninit::<T>::uninit();
        let dst_ptr = value.as_mut_ptr();

        dst_ptr.write(src_ptr.read());
        value.assume_init()
    }

    pub(crate) unsafe fn take_slice(
        &self,
        begin_idx: usize,
        len: usize,
    ) -> impl ExactSizeIterator<Item = T> {
        let vec = &mut *self.vec.get();
        let end_idx = (begin_idx + len).min(vec.len());
        let len = end_idx - begin_idx;

        let ptr = vec.as_mut_ptr().add(begin_idx);
        let vec = Vec::from_raw_parts(ptr, len, 0);
        vec.into_iter()
    }

    unsafe fn split_off_right(&self, left_len: usize) -> Vec<T> {
        debug_assert!(left_len <= self.vec_len);

        let man_vec = &mut *self.vec.get();
        let mut left_vec: Vec<T> = ManuallyDrop::take(man_vec);
        let right_vec = left_vec.split_off(left_len);
        *man_vec = ManuallyDrop::new(left_vec);
        return right_vec;
    }
}

impl<T: Send + Sync> From<Vec<T>> for ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T: Send + Sync> AtomicIter<T> for ConIterOfVec<T> {
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    #[inline(always)]
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter().fetch_and_add(number_to_fetch);
        match begin_idx.cmp(&self.initial_len()) {
            Ordering::Less => Some(begin_idx),
            _ => None,
        }
    }

    fn get(&self, item_idx: usize) -> Option<T> {
        match item_idx.cmp(&self.vec_len) {
            // SAFETY: only one thread can access the `item_idx`-th position and `item_idx` is in bounds
            Ordering::Less => Some(unsafe { self.take_one(item_idx) }),
            _ => None,
        }
    }

    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(n)
            .unwrap_or(self.initial_len());
        let end_idx = (begin_idx + n).min(self.initial_len()).max(begin_idx);

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let values = unsafe { self.take_slice(begin_idx, n) };
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn early_exit(&self) {
        self.counter().store(self.vec_len)
    }
}

impl<T: Send + Sync> AtomicIterWithInitialLen<T> for ConIterOfVec<T> {
    #[inline(always)]
    fn initial_len(&self) -> usize {
        self.vec_len
    }
}

unsafe impl<T: Send + Sync> Sync for ConIterOfVec<T> {}

unsafe impl<T: Send + Sync> Send for ConIterOfVec<T> {}

// AtomicIter -> ConcurrentIter

impl<T: Send + Sync> ConcurrentIter for ConIterOfVec<T> {
    type Item = T;

    type BufferedIter = BufferedVec<T>;

    type SeqIter = std::vec::IntoIter<T>;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    /// let con_iter = vec.into_con_iter();
    ///
    /// std::thread::scope(|s| {
    ///     s.spawn(|| {
    ///         for _ in 0..42 {
    ///             _ = con_iter.next();
    ///         }
    ///
    ///         let mut buffered = con_iter.buffered_iter(32);
    ///         let _chunk = buffered.next().unwrap();
    ///     });
    /// });
    ///
    /// let num_used = 42 + 32;
    ///
    /// // converts the remaining elements into a sequential iterator
    /// let seq_iter = con_iter.into_seq_iter();
    ///
    /// assert_eq!(seq_iter.len(), 1024 - num_used);
    /// for (i, x) in seq_iter.enumerate() {
    ///     assert_eq!(x, (num_used + i).to_string());
    /// }
    /// ```
    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter().current();
        let remaining_vec = unsafe { self.split_off_right(current.min(self.vec_len)) };
        remaining_vec.into_iter()
    }

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextChunk<Self::Item, impl ExactSizeIterator<Item = Self::Item>>> {
        self.fetch_n(chunk_size)
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIter<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIter::new(buffered_iter, self)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        let current = <Self as AtomicIter<_>>::counter(self).current();
        let initial_len = <Self as AtomicIterWithInitialLen<_>>::initial_len(self);
        let len = match current.cmp(&initial_len) {
            std::cmp::Ordering::Less => initial_len - current,
            _ => 0,
        };
        Some(len)
    }

    fn skip_to_end(&self) {
        self.early_exit()
    }
}
