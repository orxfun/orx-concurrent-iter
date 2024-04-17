use crate::{
    iter::{atomic_counter::AtomicCounter, atomic_iter::AtomicIter},
    NextChunk, NextMany,
};
use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    sync::atomic::{self, AtomicBool},
};

/// A regular `Iter: Iterator` ascended to the concurrent programs with use of atomics.
///
/// Since `ConIterOfIter` can wrap up any `Iterator` and enable concurrent iteration,
/// it might be considered as the most general `ConcurrentIter` implementation.
///
/// In performance critical scenarios and whenever possible, it might be preferable to use a more specific implementation such as [`crate::ConIterOfSlice`].
#[derive(Debug)]
pub struct ConIterOfIter<T: Send + Sync, Iter>
where
    Iter: Iterator<Item = T>,
{
    iter: UnsafeCell<Iter>,
    reserved_counter: AtomicCounter,
    yielded_counter: AtomicCounter,
    completed: AtomicBool,
}

impl<T: Send + Sync, Iter> ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    /// Creates a concurrent iterator for the given `iter`.
    pub fn new(iter: Iter) -> Self {
        Self {
            iter: iter.into(),
            reserved_counter: AtomicCounter::new(),
            yielded_counter: AtomicCounter::new(),
            completed: false.into(),
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    unsafe fn mut_iter(&self) -> &mut Iter {
        unsafe { &mut *self.iter.get() }
    }
}

impl<T: Send + Sync, Iter> From<Iter> for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    fn from(iter: Iter) -> Self {
        Self::new(iter)
    }
}

impl<T: Send + Sync, Iter> AtomicIter for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    fn counter(&self) -> &AtomicCounter {
        &self.reserved_counter
    }

    fn get(&self, item_idx: usize) -> Option<Self::Item> {
        loop {
            let yielded_count = self.yielded_counter.current();
            match item_idx.cmp(&yielded_count) {
                // item_idx==yielded_count => it is our job to provide the item
                Ordering::Equal => {
                    // SAFETY: no other thread has the valid condition to iterate, they are waiting
                    let next = unsafe { self.mut_iter() }.next();
                    return if next.is_some() {
                        _ = self.yielded_counter.fetch_and_increment();
                        next
                    } else {
                        self.completed.store(true, atomic::Ordering::Relaxed);
                        None
                    };
                }

                Ordering::Less => return None,

                // item_idx > yielded_count => we need the other items to be yielded
                Ordering::Greater => {
                    if self.completed.load(atomic::Ordering::Relaxed) {
                        return None;
                    }
                }
            }
        }
    }

    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item> {
        let begin_idx = self.counter().fetch_and_add(n);

        let idx_range = begin_idx..(begin_idx + n);
        let values = idx_range
            .map(|i| self.get(i))
            .take_while(|x| x.is_some())
            .map(|x| x.expect("is-some is checked"));

        NextMany { begin_idx, values }
    }
}

unsafe impl<T: Send + Sync, Iter> Sync for ConIterOfIter<T, Iter> where Iter: Iterator<Item = T> {}

unsafe impl<T: Send + Sync, Iter> Send for ConIterOfIter<T, Iter> where Iter: Iterator<Item = T> {}
