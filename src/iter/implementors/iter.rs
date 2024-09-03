use crate::{
    iter::buffered::iter::BufferIter, next::NextChunk, ConIterOfIterX, ConcurrentIter,
    ConcurrentIterX, Next,
};
use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

const COMPLETED: usize = usize::MAX;
const IS_MUTATING: usize = usize::MAX - 1;

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
    counter: AtomicUsize,
    yielded_counter: AtomicUsize,
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
            counter: 0.into(),
            yielded_counter: 0.into(),
        }
    }

    pub(crate) fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        match number_to_fetch {
            0 => None,
            _ => {
                let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
                loop {
                    match self.try_get_handle(begin_idx) {
                        Ok(_) => return Some(begin_idx),
                        Err(COMPLETED) => return None,
                        _ => {}
                    }
                }
            }
        }
    }

    fn get(&self, item_idx: usize) -> Option<T> {
        loop {
            match self.try_get_handle(item_idx) {
                Ok(_) => {
                    let next = unsafe { &mut *self.iter.get() }.next();

                    match next.is_some() {
                        true => self.release_handle(item_idx + 1),
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

    fn try_get_handle(&self, begin_idx: usize) -> Result<usize, usize> {
        self.yielded_counter.compare_exchange(
            begin_idx,
            IS_MUTATING,
            Ordering::Acquire,
            Ordering::Relaxed,
        )
    }

    pub(crate) fn release_handle(&self, num_taken: usize) {
        match self.yielded_counter.compare_exchange(
            IS_MUTATING,
            num_taken,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {}
            Err(e) => assert_eq!(e, COMPLETED, "Failed to update ConIterOfIter state"),
        }
    }

    pub(crate) fn release_handle_complete(&self) {
        match self.yielded_counter.compare_exchange(
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

impl<T: Send + Sync, Iter> std::fmt::Debug for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::helpers::fmt_iter(f, "ConIterOfIter", self.initial_len, &self.counter)
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

unsafe impl<T: Send + Sync, Iter> Sync for ConIterOfIter<T, Iter> where Iter: Iterator<Item = T> {}

unsafe impl<T: Send + Sync, Iter> Send for ConIterOfIter<T, Iter> where Iter: Iterator<Item = T> {}

// AtomicIter -> ConcurrentIter

impl<T: Send + Sync, Iter> ConcurrentIterX for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    type SeqIter = Iter;

    type BufferedIterX = BufferIter<T, Iter>;

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

    fn next_chunk_x(&self, chunk_size: usize) -> Option<impl ExactSizeIterator<Item = Self::Item>> {
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
                        true => self.release_handle(end_idx),
                        false => self.release_handle_complete(),
                    }

                    let values = buffer.into_iter();
                    Some(values)
                }
            },
        }
    }

    fn next(&self) -> Option<Self::Item> {
        let idx = self.counter.fetch_add(1, Ordering::Acquire);
        self.get(idx)
    }

    #[inline(always)]
    fn try_get_len(&self) -> Option<usize> {
        match self.yielded_counter.load(Ordering::SeqCst) == COMPLETED {
            true => Some(0),
            false => self.initial_len.map(|initial_len| {
                let current = self.counter.load(Ordering::Acquire);
                initial_len.saturating_sub(current)
            }),
        }
    }

    #[inline(always)]
    fn skip_to_end(&self) {
        self.yielded_counter.store(COMPLETED, Ordering::SeqCst);
    }
}

impl<T: Send + Sync, Iter> ConcurrentIter for ConIterOfIter<T, Iter>
where
    Iter: Iterator<Item = T>,
{
    type BufferedIter = Self::BufferedIterX;

    #[inline(always)]
    #[allow(refining_impl_trait)]
    fn into_concurrent_iter_x(self) -> ConIterOfIterX<T, Iter>
    where
        Self: Sized,
    {
        let iter = self.iter.into_inner();
        ConIterOfIterX::new(iter)
    }

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
                        true => self.release_handle(end_idx),
                        false => self.release_handle_complete(),
                    }

                    let values = buffer.into_iter();
                    Some(NextChunk { begin_idx, values })
                }
            },
        }
    }
}
