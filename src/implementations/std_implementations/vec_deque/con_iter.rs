use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    implementations::jagged_arrays::{
        AsRawSlice, ConIterJaggedRef, JaggedIndex, JaggedIndexer, RawJaggedRef,
    },
};
use alloc::vec;
use alloc::vec::Vec;
use core::iter::Skip;
use orx_pseudo_default::PseudoDefault;
use std::collections::VecDeque;

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
    T: Send + Sync,
{
    vec_deque: &'a VecDeque<T>,
    _slices_vec: Vec<&'a [T]>,
    con_iter: ConIterCore<'a, T>,
}

impl<'a, T> ConIterVecDequeRef<'a, T>
where
    T: Send + Sync,
{
    pub(super) fn new(vec_deque: &'a VecDeque<T>) -> Self {
        let (a, b) = vec_deque.as_slices();
        let slices_vec = vec![a, b];
        let slices = unsafe { core::slice::from_raw_parts(slices_vec.as_ptr(), 2) };
        let jagged = RawJaggedRef::new(slices, VecDequeSlicesIndexer, Some(vec_deque.len()));
        let con_iter = ConIterCore::new(jagged, 0);

        ConIterVecDequeRef {
            vec_deque,
            _slices_vec: slices_vec,
            con_iter,
        }
    }
}

type ConIterCore<'a, T> = ConIterJaggedRef<'a, T, &'a [T], VecDequeSlicesIndexer>;

#[derive(Clone)]
pub struct VecDequeSlicesIndexer;

impl PseudoDefault for VecDequeSlicesIndexer {
    fn pseudo_default() -> Self {
        Self
    }
}

impl JaggedIndexer for VecDequeSlicesIndexer {
    fn jagged_index<T>(
        &self,
        total_len: usize,
        arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> Option<JaggedIndex> {
        (flat_index <= total_len)
            .then_some(unsafe { self.jagged_index_unchecked(arrays, flat_index) })
    }

    unsafe fn jagged_index_unchecked<T>(
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
    T: Send + Sync,
{
    type Item = <ConIterCore<'a, T> as ConcurrentIter>::Item;

    type SequentialIter = Skip<std::collections::vec_deque::Iter<'a, T>>;

    type ChunkPuller<'i>
        = <ConIterCore<'a, T> as ConcurrentIter>::ChunkPuller<'i>
    where
        Self: 'i;

    fn into_seq_iter(self) -> Self::SequentialIter {
        let num_remaining = self.len();
        let skip = self.vec_deque.len().saturating_sub(num_remaining);
        self.vec_deque.iter().skip(skip)
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

    fn chunk_puller(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        self.con_iter.chunk_puller(chunk_size)
    }
}

impl<T> ExactSizeConcurrentIter for ConIterVecDequeRef<'_, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.con_iter.len()
    }
}
