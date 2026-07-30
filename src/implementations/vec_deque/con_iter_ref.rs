use super::vec_deque_ref::VecDequeRef;
use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    implementations::jagged_arrays::{
        AsRawSlice, ConIterJaggedRef, JaggedIndex, JaggedIndexer, RawJaggedRef, Slices,
    },
};
use alloc::collections::VecDeque;
use orx_pseudo_default::PseudoDefault;

/// Concurrent iterator of a reference to a [`VecDeque`].
///
/// It can be created by calling [`into_con_iter`] on a `&VecDeque`.
///
/// Alternatively, it can be created calling [`con_iter`] on `VecDeque`.
///
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
/// [`con_iter`]: crate::ConcurrentIterable::con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// // &[T]: IntoConcurrentIter
/// let vec = vec![0, 1, 2, 3];
/// let slice = &vec[1..3];
/// let con_iter = slice.into_con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
///
/// // Vec<T>: ConcurrentIterable
/// let vec = vec![1, 2];
/// let con_iter = vec.con_iter();
/// assert_eq!(con_iter.next(), Some(&1));
/// assert_eq!(con_iter.next(), Some(&2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct ConIterVecDequeRef<'a, T>
where
    T: 'a + Sync,
{
    con_iter: ConIterCore<'a, T>,
}

impl<'a, T> ConIterVecDequeRef<'a, T>
where
    T: 'a + Sync,
{
    pub(super) fn new(vec_deque_ref: &'a VecDeque<T>) -> Self {
        let len = vec_deque_ref.len();
        let vec_deque_ref = VecDequeRef::new(vec_deque_ref);
        let jagged = RawJaggedRef::new(vec_deque_ref, VecDequeSlicesIndexer, Some(len));
        let con_iter = ConIterCore::new(jagged, 0);
        Self { con_iter }
    }
}

type ConIterCore<'a, T> = ConIterJaggedRef<'a, T, VecDequeRef<'a, T>, VecDequeSlicesIndexer>;

#[derive(Clone)]
pub struct VecDequeSlicesIndexer;

impl PseudoDefault for VecDequeSlicesIndexer {
    fn pseudo_default() -> Self {
        Self
    }
}

impl JaggedIndexer for VecDequeSlicesIndexer {
    unsafe fn jagged_index_unchecked<'a, T: 'a>(
        &self,
        arrays: &impl Slices<'a, T>,
        flat_index: usize,
    ) -> JaggedIndex {
        let first = unsafe { arrays.slice_at_unchecked(0) };
        match flat_index < first.len() {
            true => JaggedIndex::new(0, flat_index),
            false => JaggedIndex::new(1, flat_index - first.len()),
        }
    }

    unsafe fn jagged_index_unchecked_from_slice<'a, T: 'a>(
        &self,
        arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        match flat_index < arrays[0].length() {
            true => JaggedIndex::new(0, flat_index),
            false => JaggedIndex::new(1, flat_index - arrays[0].length()),
        }
    }
}

impl<'a, T> ConcurrentIter for ConIterVecDequeRef<'a, T>
where
    T: 'a + Sync,
{
    type Item = <ConIterCore<'a, T> as ConcurrentIter>::Item;

    type SequentialIter = <ConIterCore<'a, T> as ConcurrentIter>::SequentialIter;

    type ChunkPuller<'i>
        = <ConIterCore<'a, T> as ConcurrentIter>::ChunkPuller<'i>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        self.con_iter.into_seq_iter()
    }

    fn skip_to_end(&self) {
        self.con_iter.skip_to_end();
    }

    fn next(&self) -> Option<Self::Item> {
        self.con_iter.next()
    }

    fn next_with_idx(&self) -> Option<(usize, Self::Item)> {
        self.con_iter.next_with_idx()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.con_iter.size_hint()
    }

    fn is_completed_when_none_returned(&self) -> bool {
        true
    }

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        self.con_iter.chunk_puller(chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterVecDequeRef<'_, T>
where
    T: Sync,
{
    fn len(&self) -> usize {
        self.con_iter.len()
    }
}
