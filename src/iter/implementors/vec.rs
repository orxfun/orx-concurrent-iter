use crate::{
    iter::atomic_iter::AtomicIterWithInitialLen, AtomicCounter, AtomicIter, NextChunk,
    NextManyExact,
};
use std::{cell::UnsafeCell, cmp::Ordering};

/// A concurrent iterator over a vector, consuming the vector and yielding its elements.
#[derive(Debug)]
pub struct ConIterOfVec<T: Send + Sync + Default> {
    vec: UnsafeCell<Vec<T>>,
    vec_len: usize,
    counter: AtomicCounter,
}

impl<T: Send + Sync + Default> ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            vec_len: vec.len(),
            vec: vec.into(),
            counter: AtomicCounter::new(),
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    unsafe fn mut_vec(&self) -> &mut Vec<T> {
        unsafe { &mut *self.vec.get() }
    }
}

impl<T: Send + Sync + Default> From<Vec<T>> for ConIterOfVec<T> {
    /// Consumes and creates a concurrent iterator of the given `vec`.
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T: Send + Sync + Default> AtomicIter for ConIterOfVec<T> {
    type Item = T;

    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    fn get(&self, item_idx: usize) -> Option<Self::Item> {
        match item_idx.cmp(&self.vec_len) {
            Ordering::Less => {
                // SAFETY: only one thread can access the `item_idx`-th position and `item_idx` is in bounds
                let vec = unsafe { self.mut_vec() };
                let value = std::mem::take(&mut vec[item_idx]);
                Some(value)
            }
            _ => None,
        }
    }

    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n_with_exact_len(n)
    }
}

impl<T: Send + Sync + Default> AtomicIterWithInitialLen for ConIterOfVec<T> {
    fn initial_len(&self) -> usize {
        self.vec_len
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>> {
        let vec = unsafe { self.mut_vec() };

        let begin_idx = self.counter().fetch_and_add(n);
        let end_idx = (begin_idx + n).min(self.vec_len).max(begin_idx);
        let idx_range = begin_idx..end_idx;
        let values = idx_range.map(|i| unsafe { get_unchecked(vec, i) });

        NextManyExact { begin_idx, values }
    }
}

#[inline(always)]
unsafe fn get_unchecked<T: Default>(vec: &mut [T], item_idx: usize) -> T {
    std::mem::take(&mut vec[item_idx])
}

unsafe impl<T: Send + Sync + Default> Sync for ConIterOfVec<T> {}

unsafe impl<T: Send + Sync + Default> Send for ConIterOfVec<T> {}
