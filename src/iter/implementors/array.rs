use crate::{
    iter::{
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
        buffered::{
            array::BufferedArray, buffered_chunk::BufferedChunk, buffered_iter::BufferedIter,
        },
    },
    next::NextChunk,
    AtomicCounter, ConcurrentIter, Next,
};
use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    mem::{ManuallyDrop, MaybeUninit},
};

/// A concurrent iterator over an array, consuming the array and yielding its elements.
#[derive(Debug)]
pub struct ConIterOfArray<const N: usize, T: Send + Sync> {
    array: UnsafeCell<ManuallyDrop<[T; N]>>,
    counter: AtomicCounter,
}

impl<const N: usize, T: Send + Sync> Drop for ConIterOfArray<N, T> {
    fn drop(&mut self) {
        let current = self.counter().current();
        if current <= N {
            let _remaining_vec_to_be_dropped = unsafe { self.split_off_right(current) };
        }
    }
}

impl<const N: usize, T: Send + Sync> ConIterOfArray<N, T> {
    /// Consumes and creates a concurrent iterator of the given `array`.
    pub fn new(array: [T; N]) -> Self {
        Self {
            array: ManuallyDrop::new(array).into(),
            counter: AtomicCounter::new(),
        }
    }

    unsafe fn take_one(&self, item_idx: usize) -> T {
        let array = &mut *self.array.get();
        let src_ptr = array.as_mut_ptr().add(item_idx);

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
        let array = &mut *self.array.get();
        let end_idx = (begin_idx + len).min(array.len());
        let len = end_idx - begin_idx;

        let ptr = array.as_mut_ptr().add(begin_idx);
        let vec = Vec::from_raw_parts(ptr, len, 0);
        vec.into_iter()
    }

    unsafe fn split_off_right(&self, left_len: usize) -> Vec<T> {
        debug_assert!(left_len <= N);

        let man_array = &mut *self.array.get();
        let mut array = ManuallyDrop::take(man_array);

        let mut vec = Vec::from_raw_parts(array.as_mut_ptr(), N, 0);
        let right_vec = vec.split_off(left_len);

        *man_array = ManuallyDrop::new(array);
        right_vec
    }
}

impl<const N: usize, T: Send + Sync> From<[T; N]> for ConIterOfArray<N, T> {
    /// Consumes and creates a concurrent iterator of the given `array`.
    fn from(array: [T; N]) -> Self {
        Self::new(array)
    }
}

impl<const N: usize, T: Send + Sync> AtomicIter<T> for ConIterOfArray<N, T> {
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
        match item_idx.cmp(&N) {
            // SAFETY: only one thread can access the `item_idx`-th position and `item_idx` is in bounds
            Ordering::Less => Some(unsafe { self.take_one(item_idx) }),
            _ => None,
        }
    }

    #[inline(always)]
    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        let begin_idx = self
            .progress_and_get_begin_idx(n)
            .unwrap_or(self.initial_len());
        let end_idx = (begin_idx + n).min(N).max(begin_idx);

        match begin_idx.cmp(&end_idx) {
            Ordering::Equal => None,
            _ => {
                let values = unsafe { self.take_slice(begin_idx, n) };
                Some(NextChunk { begin_idx, values })
            }
        }
    }

    fn early_exit(&self) {
        self.counter().store(N)
    }
}

impl<const N: usize, T: Send + Sync> AtomicIterWithInitialLen<T> for ConIterOfArray<N, T> {
    #[inline(always)]
    fn initial_len(&self) -> usize {
        N
    }
}

unsafe impl<const N: usize, T: Send + Sync> Sync for ConIterOfArray<N, T> {}

unsafe impl<const N: usize, T: Send + Sync> Send for ConIterOfArray<N, T> {}

// AtomicIter -> ConcurrentIter

impl<const N: usize, T: Send + Sync> ConcurrentIter for ConIterOfArray<N, T> {
    type Item = T;

    type BufferedIter = BufferedArray<N, T>;

    type SeqIter = std::vec::IntoIter<T>;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let mut array = [0; 1024];
    /// for (i, x) in array.iter_mut().enumerate() {
    ///     *x = i;
    /// }
    /// let con_iter = array.into_con_iter();
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
    ///     assert_eq!(x, num_used + i);
    /// }
    /// ```
    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter().current();
        let remaining_vec = unsafe { self.split_off_right(current.min(N)) };
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
