use super::{
    iter_cell::IterCell,
    mut_handle::{AtomicState, MutHandle, COMPLETED},
};
use crate::concurrent_iter::ConcurrentIter;
use core::sync::atomic::Ordering;

pub struct ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
    iter: IterCell<I::Item, I>,
    initial_len: Option<usize>,
    state: AtomicState,
}

unsafe impl<I> Sync for ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
}

unsafe impl<I> Send for ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
}

impl<I> Default for ConIterOfIter<I>
where
    I: Iterator + Default,
    I::Item: Send + Sync,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I> ConIterOfIter<I>
where
    I: Iterator,
    I::Item: Send + Sync,
{
    pub(super) fn new(iter: I) -> Self {
        let initial_len = match iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(lower),
            _ => None,
        };

        Self {
            iter: iter.into(),
            initial_len,
            state: 0.into(),
        }
    }

    fn get_handle(&self) -> Option<MutHandle<'_>> {
        MutHandle::get_handle(&self.state)
    }

    /// Pulls and writes chunk-size (`buffer.len()`) elements from the iterator into the given `buffer` starting from position 0.
    ///
    /// Returns the pair of (begin_idx, num_taken):
    ///
    /// * begin_idx: index of the first taken item.
    /// * num_taken: number of items pulled from the iterator; the method tries to pull `buffer.len()` items, however, might stop
    ///   early if the iterator is completely consumed.
    pub(super) fn next_chunk_to_buffer(&self, buffer: &mut [Option<I::Item>]) -> (usize, usize) {
        self.get_handle()
            .map(|handle| self.iter.next_chunk_to_buffer(handle, buffer))
            .unwrap_or((0, 0))
    }
}

// impl<I> ConcurrentIter for ConIterOfIter<I>
// where
//     I: Iterator,
//     I::Item: Send + Sync,
// {
//     type Item = I::Item;

//     type SeqIter = I;

//     type ChunkPuller<'i>
//         = ChunkPullerOfIter<'i, I, E>
//     where
//         Self: 'i;

//     fn into_seq_iter(self) -> Self::SeqIter {
//         self.iter.into_inner()
//     }

//     fn skip_to_end(&self) {
//         self.state.store(COMPLETED, Ordering::SeqCst);
//     }

//     fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
//         self.get_handle()
//             .and_then(|handle| self.iter.next::<E>(handle))
//     }

//     fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
//         Self::ChunkPuller::new(self, chunk_size)
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         match self.get_handle() {
//             Some(handle) => self.iter.size_hint(handle),
//             None => (0, Some(0)),
//         }
//     }
// }
