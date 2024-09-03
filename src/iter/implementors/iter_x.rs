use crate::iter::{
    atomic_counter::AtomicCounter,
    buffered::{buffered_chunk::BufferedChunkX, buffered_iter::BufferedIterX, iter_x::BufferIterX},
    con_iter_x::ConcurrentIterX,
};
use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU8, Ordering},
};

/// A regular `Iter: Iterator` ascended to the concurrent programs with use of atomics.
///
/// Since `ConIterOfIter` can wrap up any `Iterator` and enable concurrent iteration,
/// it might be considered as the most general `ConcurrentIter` implementation.
///
/// In performance critical scenarios and whenever possible, it might be preferable to use a more specific implementation such as [`crate::ConIterOfSlice`].
pub struct ConIterOfIterX<T: Send + Sync, Iter>
where
    Iter: Iterator<Item = T>,
{
    pub(crate) iter: UnsafeCell<Iter>,
    initial_len: Option<usize>,
    reserved_counter: AtomicCounter,
    is_mutating: AtomicU8,
}

type State = u8;
const AVAILABLE: State = 0;
const IS_MUTATING: State = 1;
const COMPLETED: State = 2;

impl<T: Send + Sync, Iter> ConIterOfIterX<T, Iter>
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
            is_mutating: AVAILABLE.into(),
        }
    }

    pub(crate) fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        match number_to_fetch {
            0 => None,
            _ => {
                let begin_idx = self.reserved_counter.fetch_and_add(number_to_fetch);

                loop {
                    match self.try_get_handle() {
                        Ok(()) => return Some(begin_idx),
                        Err(COMPLETED) => return None,
                        _ => {}
                    }
                }
            }
        }
    }

    fn get(&self, _item_idx: usize) -> Option<T> {
        loop {
            match self.try_get_handle() {
                Ok(()) => {
                    let next = unsafe { &mut *self.iter.get() }.next();

                    match next.is_some() {
                        true => self.release_handle(),
                        false => self.release_handle_complete(),
                    }

                    return next;
                }
                Err(COMPLETED) => return None,
                _ => {}
            }
        }
    }

    // handles

    fn try_get_handle(&self) -> Result<(), State> {
        self.is_mutating
            .compare_exchange(AVAILABLE, IS_MUTATING, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| ())
    }

    pub(crate) fn release_handle(&self) {
        match self.is_mutating.compare_exchange(
            IS_MUTATING,
            AVAILABLE,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {}
            Err(e) => assert_eq!(e, COMPLETED, "Failed to update ConIterOfIter state"),
        }
    }

    pub(crate) fn release_handle_complete(&self) {
        match self.is_mutating.compare_exchange(
            IS_MUTATING,
            COMPLETED,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {}
            Err(e) => assert_eq!(e, COMPLETED, "Failed to update ConIterOfIter state"),
        }
    }
}

impl<T: Send + Sync, Iter> std::fmt::Debug for ConIterOfIterX<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::helpers::fmt_iter(
            f,
            "ConIterOfIterX",
            self.initial_len,
            &self.reserved_counter,
        )
    }
}

impl<T: Send + Sync, Iter> From<Iter> for ConIterOfIterX<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    fn from(iter: Iter) -> Self {
        Self::new(iter)
    }
}

unsafe impl<T: Send + Sync, Iter> Sync for ConIterOfIterX<T, Iter> where Iter: Iterator<Item = T> {}

unsafe impl<T: Send + Sync, Iter> Send for ConIterOfIterX<T, Iter> where Iter: Iterator<Item = T> {}

// AtomicIter -> ConcurrentIter

impl<T: Send + Sync, Iter> ConcurrentIterX for ConIterOfIterX<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    type BufferedIter = BufferIterX<T, Iter>;

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

    fn next(&self) -> Option<Self::Item> {
        let idx = self.reserved_counter.fetch_and_increment();
        self.get(idx)
    }

    fn next_chunk(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = Self::Item>> {
        match chunk_size {
            0 => None,
            _ => match self.progress_and_get_begin_idx(chunk_size) {
                None => None,
                Some(begin_idx) => {
                    let iter = unsafe { &mut *self.iter.get() };

                    let end_idx = begin_idx + chunk_size;

                    let buffer: Vec<_> = (begin_idx..end_idx)
                        .map(|_| iter.next())
                        .take_while(|x| x.is_some())
                        .map(|x| x.expect("must be some"))
                        .collect();

                    match buffer.len() == chunk_size {
                        true => self.release_handle(),
                        false => self.release_handle_complete(),
                    }

                    let values = buffer.into_iter();
                    Some(values)
                }
            },
        }
    }

    fn skip_to_end(&self) {
        self.is_mutating.store(COMPLETED, Ordering::SeqCst);
    }

    fn try_get_len(&self) -> Option<usize> {
        match self.is_mutating.load(Ordering::SeqCst) == COMPLETED {
            true => Some(0),
            false => self.initial_len.map(|initial_len| {
                let current = self.reserved_counter.current();
                initial_len.saturating_sub(current)
            }),
        }
    }

    fn buffered_iter(&self, chunk_size: usize) -> BufferedIterX<Self::Item, Self::BufferedIter> {
        let buffered_iter = Self::BufferedIter::new(chunk_size);
        BufferedIterX::new(buffered_iter, self)
    }
}
