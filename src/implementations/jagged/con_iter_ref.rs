use super::{
    chunk_puller_owned::ChunkPullerJaggedOwned, chunk_puller_ref::ChunkPullerJaggedRef,
    raw_jagged::RawJagged, raw_jagged_iter_owned::RawJaggedIterOwned,
    raw_jagged_slice_iter_owned::RawJaggedSliceIterOwned,
    raw_jagged_slice_iter_ref::RawJaggedSliceIterRef,
};
use crate::{ConcurrentIter, ExactSizeConcurrentIter};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
    jagged: &'a RawJagged<T, X>,
    counter: AtomicUsize,
}

unsafe impl<'a, T, X> Sync for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
}

unsafe impl<'a, T, X> Send for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
}

impl<'a, T, X> ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
    pub(crate) fn new(jagged: &'a RawJagged<T, X>, begin: usize) -> Self {
        Self {
            jagged,
            counter: begin.into(),
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.jagged.len() {
            true => Some(begin_idx),
            false => None,
        }
    }

    pub(super) fn progress_and_get_iter(
        &self,
        chunk_size: usize,
    ) -> Option<(usize, RawJaggedSliceIterRef<'a, T>)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size)
                    .min(self.jagged.len())
                    .max(begin_idx);
                let slice = self.jagged.slice(begin_idx, end_idx);
                let iter = slice.into_iter_ref();
                (begin_idx, iter)
            })
    }
}

impl<'a, T, X> ConcurrentIter for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
    type Item = &'a T;

    type SequentialIter = RawJaggedSliceIterRef<'a, T>;

    type ChunkPuller<'i>
        = ChunkPullerJaggedRef<'i, 'a, T, X>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let num_taken = self.counter.load(Ordering::Acquire).min(self.jagged.len());
        let slice = self.jagged.slice_from(num_taken);
        slice.into_iter_ref()
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.jagged.len(), Ordering::Acquire);
    }

    fn next(&self) -> Option<Self::Item> {
        // self.progress_and_get_begin_idx(1)
        //     .and_then(|idx| self.jagged.take(idx))
        todo!()
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        // self.progress_and_get_begin_idx(1)
        //     .and_then(|idx| self.jagged.take(idx).map(|value| (idx, value)))
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_taken = self.counter.load(Ordering::Acquire);
        let remaining = self.jagged.len().saturating_sub(num_taken);
        (remaining, Some(remaining))
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}

impl<'a, T, X> ExactSizeConcurrentIter for ConIterJaggedRef<'a, T, X>
where
    T: Send + Sync,
    X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
{
    fn len(&self) -> usize {
        let num_taken = self.counter.load(Ordering::Acquire);
        self.jagged.len().saturating_sub(num_taken)
    }
}

// impl<'a, T, X> Drop for  ConIterJaggedRef<'a, T, X>
// where
//     T: Send + Sync,
//     X: Fn(usize) -> [usize; 2] + Clone + Send + Sync,
// {
//     fn drop(&mut self) {
//         if self.jagged.num_taken().is_some() {
//             let num_taken = self.counter.load(Ordering::Acquire);
//             self.jagged.set_num_taken(Some(num_taken));
//         }
//     }
// }
