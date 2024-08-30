use super::{iter_mut_handle::IterMutHandle, iter_mut_states::*};
use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        atomic_iter::AtomicIter,
        buffered::{buffered_chunk::BufferedChunk, buffered_iter::BufferedIter, iter::BufferIter},
    },
    next::NextChunk,
    ConcurrentIter, Next,
};
use std::{
    cell::UnsafeCell,
    cmp::Ordering,
    sync::atomic::{self, AtomicU8},
};

/// A regular `Iter: Iterator` ascended to the concurrent programs with use of atomics.
///
/// Since `ConIterOfIter` can wrap up any `Iterator` and enable concurrent iteration,
/// it might be considered as the most general `ConcurrentIter` implementation.
///
/// In performance critical scenarios and whenever possible, it might be preferable to use a more specific implementation such as [`crate::ConIterOfSlice`].
pub struct ConIterOfIter<T: Send + Sync, Iter>
where
    Iter: Iterator<Item = T>,
{
    pub(crate) iter: UnsafeCell<Iter>,
    initial_len: Option<usize>,
    reserved_counter: AtomicCounter,
    yielded_counter: AtomicCounter,
    state: AtomicU8,
}

impl<T: Send + Sync, Iter> ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    /// Creates a concurrent iterator for the given `iter`.
    pub fn new(iter: Iter) -> Self {
        let initial_len = match iter.size_hint() {
            (_, None) => None,
            (lower, Some(upper)) if lower == upper => Some(lower),
            _ => None,
        };

        Self {
            iter: iter.into(),
            initial_len,
            reserved_counter: AtomicCounter::new(),
            yielded_counter: AtomicCounter::new(),
            state: AVAILABLE.into(),
        }
    }

    #[inline(always)]
    pub(crate) fn progress_yielded_counter(&self, num_yielded: usize) -> usize {
        self.yielded_counter.fetch_and_add(num_yielded)
    }

    pub(crate) fn mut_handle(&self) -> Option<IterMutHandle> {
        IterMutHandle::spin_get(&self.state)
    }
}

impl<T: Send + Sync, Iter> std::fmt::Debug for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::helpers::fmt_iter(f, "ConIterOfIter", self.initial_len, &self.yielded_counter)
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

    #[inline(always)]
    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        match number_to_fetch {
            0 => None,
            _ => {
                let begin_idx = self.counter().fetch_and_add(number_to_fetch);

                loop {
                    let yielded_count = self.yielded_counter.current();
                    match begin_idx.cmp(&yielded_count) {
                        // begin_idx==yielded_count => it is our job to provide the items
                        Ordering::Equal => return Some(begin_idx),

                        Ordering::Less => return None,

                        // begin_idx > yielded_count => we need the other items to be yielded
                        Ordering::Greater => {
                            if self.state.load(atomic::Ordering::Relaxed) == COMPLETED {
                                return None;
                            }
                        }
                    }
                }
            }
        }
    }

    fn get(&self, item_idx: usize) -> Option<T> {
        loop {
            let yielded_count = self.yielded_counter.current();
            match item_idx.cmp(&yielded_count) {
                // item_idx==yielded_count => it is our job to provide the item
                Ordering::Equal => {
                    // SAFETY: no other thread has the valid condition to iterate, they are waiting
                    let next = {
                        self.mut_handle()
                            .and_then(|_h| unsafe { &mut *self.iter.get() }.next())
                    };

                    match next.is_some() {
                        true => {
                            _ = self.yielded_counter.fetch_and_increment();
                        }
                        false => self.state.store(COMPLETED, atomic::Ordering::Release),
                    };
                    return next;
                }

                Ordering::Less => return None,

                // item_idx > yielded_count => we need the other items to be yielded
                Ordering::Greater => {
                    if self.state.load(atomic::Ordering::Relaxed) == COMPLETED {
                        return None;
                    }
                }
            }
        }
    }

    fn fetch_n(&self, n: usize) -> Option<NextChunk<T, impl ExactSizeIterator<Item = T>>> {
        self.progress_and_get_begin_idx(n).and_then(|begin_idx| {
            // SAFETY: no other thread has the valid condition to iterate, they are waiting
            let buffer = self
                .mut_handle()
                .map(|_h| {
                    let iter = unsafe { &mut *self.iter.get() };
                    let end_idx = begin_idx + n;
                    (begin_idx..end_idx)
                        .map(|_| iter.next())
                        .take_while(|x| x.is_some())
                        .map(|x| x.expect("is_some is checked"))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            match buffer.len() {
                0 => {
                    self.state.store(COMPLETED, atomic::Ordering::SeqCst);
                    let older_count = self.progress_yielded_counter(n);
                    assert_eq!(older_count, begin_idx);
                    None
                }
                _ => {
                    let values = buffer.into_iter();
                    let older_count = self.progress_yielded_counter(n);
                    assert_eq!(older_count, begin_idx);
                    Some(NextChunk { begin_idx, values })
                }
            }
        })
    }

    fn early_exit(&self) {
        self.counter().store(usize::MAX);
        self.state.store(COMPLETED, atomic::Ordering::SeqCst);
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

    type BufferedIter = BufferIter<T, Iter>;

    type SeqIter = Iter;

    /// Converts the concurrent iterator back to the original wrapped type which is the source of the elements to be iterated.
    /// Already progressed elements are skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_iter::*;
    ///
    /// let iter = (0..1024).map(|x| x.to_string());
    /// let con_iter = iter.into_con_iter();
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
        self.iter.into_inner()
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
        match self.state.load(atomic::Ordering::SeqCst) == COMPLETED {
            true => Some(0),
            false => self.initial_len.map(|initial_len| {
                let current = <Self as AtomicIter<_>>::counter(self).current();
                match current.cmp(&initial_len) {
                    std::cmp::Ordering::Less => initial_len - current,
                    _ => 0,
                }
            }),
        }
    }

    fn skip_to_end(&self) {
        self.early_exit()
    }
}
