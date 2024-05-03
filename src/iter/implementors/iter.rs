use crate::{
    iter::{atomic_counter::AtomicCounter, atomic_iter::AtomicIter, default_fns},
    ConcurrentIter, Next, NextChunk, NextMany,
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

impl<T: Send + Sync, Iter> AtomicIter<T> for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    #[inline(always)]
    fn counter(&self) -> &AtomicCounter {
        &self.reserved_counter
    }

    fn get(&self, item_idx: usize) -> Option<T> {
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

    fn fetch_n(&self, n: usize) -> impl NextChunk<T> {
        let begin_idx = self.counter().fetch_and_add(n);

        loop {
            let yielded_count = self.yielded_counter.current();
            match begin_idx.cmp(&yielded_count) {
                // begin_idx==yielded_count => it is our job to provide the items
                Ordering::Equal => {
                    // SAFETY: no other thread has the valid condition to iterate, they are waiting
                    let iter = unsafe { self.mut_iter() };
                    let idx_range = begin_idx..(begin_idx + n);
                    let values = idx_range
                        .map(|_| iter.next())
                        .take_while(|x| x.is_some())
                        .map(|x| x.expect("is_some is checked"))
                        .collect::<Vec<_>>();
                    let older_count = self.yielded_counter.fetch_and_add(n);
                    assert_eq!(older_count, begin_idx);

                    return NextMany { begin_idx, values };
                }

                Ordering::Less => {
                    return NextMany {
                        begin_idx,
                        values: vec![],
                    }
                }

                // begin_idx > yielded_count => we need the other items to be yielded
                Ordering::Greater => {
                    if self.completed.load(atomic::Ordering::Relaxed) {
                        return NextMany {
                            begin_idx,
                            values: vec![],
                        };
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(T)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::any_for_each(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, T)>(&self, chunk_size: usize, f: F) {
        default_fns::for_each::any_for_each_with_ids(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, F: FnMut(B, T) -> B>(&self, chunk_size: usize, initial: B, f: F) -> B {
        default_fns::fold::any_fold(self, chunk_size, f, initial)
    }
}

unsafe impl<T: Send + Sync, Iter> Sync for ConIterOfIter<T, Iter> where Iter: Iterator<Item = T> {}

unsafe impl<T: Send + Sync, Iter> Send for ConIterOfIter<T, Iter> where Iter: Iterator<Item = T> {}

// AtomicIter -> ConcurrentIter

impl<T: Send + Sync, Iter> ConcurrentIter for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    #[inline(always)]
    fn next_id_and_value(&self) -> Option<Next<Self::Item>> {
        self.fetch_one()
    }

    #[inline(always)]
    fn next_chunk(&self, chunk_size: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n(chunk_size)
    }

    #[inline(always)]
    fn for_each_n<F: FnMut(Self::Item)>(&self, chunk_size: usize, f: F) {
        <Self as AtomicIter<_>>::for_each_n(self, chunk_size, f)
    }

    #[inline(always)]
    fn enumerate_for_each_n<F: FnMut(usize, Self::Item)>(&self, chunk_size: usize, f: F) {
        <Self as AtomicIter<_>>::enumerate_for_each_n(self, chunk_size, f)
    }

    #[inline(always)]
    fn fold<B, Fold>(&self, chunk_size: usize, neutral: B, fold: Fold) -> B
    where
        Fold: FnMut(B, Self::Item) -> B,
    {
        <Self as AtomicIter<_>>::fold(self, chunk_size, neutral, fold)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        None
    }
}
