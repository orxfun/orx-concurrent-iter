use super::mut_handle::MutHandle;
use core::{cell::UnsafeCell, marker::PhantomData};

pub struct IterCell<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: UnsafeCell<I>,
    num_taken: UnsafeCell<usize>,
    phantom: PhantomData<T>,
}

impl<T, I> From<I> for IterCell<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    fn from(iter: I) -> Self {
        Self {
            iter: iter.into(),
            num_taken: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<T, I> IterCell<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    pub fn into_inner(self) -> I {
        self.iter.into_inner()
    }

    /// Pulls the next element from the iterator and returns its enumerated value.
    ///
    /// Returns None if the iterator is completely consumed.
    /// In this case, the `handle` will finalize its state as COMPLETED when dropped.
    ///
    /// # SAFETY
    ///
    /// Only one thread can call this method at a given instant.
    /// This is satisfied by the mut handle.
    #[inline(always)]
    pub fn next(&self, mut handle: MutHandle) -> Option<T> {
        match unsafe { &mut *self.iter.get() }.next() {
            Some(item) => {
                let num_taken = unsafe { &mut *self.num_taken.get() };
                let idx = *num_taken;
                *num_taken = idx + 1;
                Some(item)
            }
            None => {
                handle.set_target_to_completed();
                None
            }
        }
    }

    /// Pulls the next element from the iterator and returns its enumerated value.
    ///
    /// Returns None if the iterator is completely consumed.
    /// In this case, the `handle` will finalize its state as COMPLETED when dropped.
    ///
    /// # SAFETY
    ///
    /// Only one thread can call this method at a given instant.
    /// This is satisfied by the mut handle.
    #[inline(always)]
    pub fn next_with_idx(&self, mut handle: MutHandle) -> Option<(usize, T)> {
        match unsafe { &mut *self.iter.get() }.next() {
            Some(item) => {
                let num_taken = unsafe { &mut *self.num_taken.get() };
                let idx = *num_taken;
                *num_taken = idx + 1;
                Some((idx, item))
            }
            None => {
                handle.set_target_to_completed();
                None
            }
        }
    }

    /// Pulls and writes chunk-size (`buffer.len()`) elements from the iterator into the given `buffer` starting from position 0.
    ///
    /// Returns the pair of (begin_idx, num_taken):
    ///
    /// * begin_idx: index of the first taken item.
    /// * num_taken: number of items pulled from the iterator; the method tries to pull `buffer.len()` items, however, might stop
    ///   early if the iterator is completely consumed.
    ///
    /// If the method returns num_taken < buffer.len(); i.e., if the iterator is completely consumed,
    /// the `handle` will finalize its state as COMPLETED when dropped.
    ///
    /// # SAFETY
    ///
    /// Only one thread can call this method at a given instant.
    /// This is satisfied by the mut handle.
    pub fn next_chunk_to_buffer(
        &self,
        mut handle: MutHandle,
        buffer: &mut [Option<T>],
    ) -> (usize, usize) {
        let num_taken = unsafe { &mut *self.num_taken.get() };
        let begin_idx = *num_taken;

        let iter = unsafe { &mut *self.iter.get() };
        let mut num_taken_now = buffer.len();

        for (i, x) in buffer.iter_mut().enumerate() {
            match iter.next() {
                Some(item) => {
                    *x = Some(item);
                }
                None => {
                    num_taken_now = i;
                    handle.set_target_to_completed();
                }
            }
        }

        *num_taken += num_taken_now;

        (begin_idx, num_taken_now)
    }

    pub fn size_hint(&self, _handle: MutHandle) -> (usize, Option<usize>) {
        let iter = unsafe { &mut *self.iter.get() };
        iter.size_hint()
    }

    pub fn len(&self, _handle: MutHandle) -> usize
    where
        I: ExactSizeIterator,
    {
        let iter = unsafe { &mut *self.iter.get() };
        iter.len()
    }
}
