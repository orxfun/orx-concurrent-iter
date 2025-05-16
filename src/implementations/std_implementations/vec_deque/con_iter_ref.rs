use crate::{
    ConcurrentIter, ExactSizeConcurrentIter,
    implementations::jagged_arrays::{
        AsRawSlice, ConIterJaggedRef, JaggedIndex, JaggedIndexer, RawJaggedRef,
    },
};
use alloc::vec;
use alloc::vec::Vec;
use core::{iter::Skip, mem::ManuallyDrop, ptr::drop_in_place};
use orx_pseudo_default::PseudoDefault;
use std::{collections::VecDeque, time::UNIX_EPOCH};

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
    slices: *const &'a [T],
    con_iter: ConIterCore<'a, T>,
}

unsafe impl<'a, T: Send + Sync> Sync for ConIterVecDequeRef<'a, T> {}

unsafe impl<'a, T: Send + Sync> Send for ConIterVecDequeRef<'a, T> {}

// impl<'a, T> Drop for ConIterVecDequeRef<'a, T>
// where
//     T: Send + Sync,
// {
//     fn drop(&mut self) {
//         let ptr = self.slices as *mut &'a [T];
//         unsafe { ptr.write(Default::default()) };
//         unsafe { ptr.add(1).write(Default::default()) };

//         let _vec_to_drop = unsafe { Vec::from_raw_parts(self.slices as *mut &'a [T], 0, 2) };
//     }
// }

impl<'a, T> ConIterVecDequeRef<'a, T>
where
    T: Send + Sync,
{
    pub(super) fn new(vec_deque: &'a VecDeque<T>) -> Self {
        let (a, b) = vec_deque.as_slices();

        let mut x = Self {
            vec_deque,
            slices: core::ptr::null(),
            con_iter: ConIterCore::new(Default::default(), 0),
        };

        let mut slices_vec = Vec::with_capacity(2);
        slices_vec.push(unsafe { core::slice::from_raw_parts(a.as_ptr(), a.len()) });
        slices_vec.push(unsafe { core::slice::from_raw_parts(b.as_ptr(), b.len()) });
        let slices_arr = ManuallyDrop::new(slices_vec);
        x.slices = slices_arr.as_ptr();
        let slices = unsafe { core::slice::from_raw_parts(x.slices, 2) };
        let jagged = RawJaggedRef::new(slices, VecDequeSlicesIndexer, Some(vec_deque.len()));
        let con_iter = ConIterCore::new(jagged, 0);
        x.con_iter = con_iter;
        x

        // let (a, b) = vec_deque.as_slices();
        // let slices_vec = vec![a, b];
        // let slices = unsafe { core::slice::from_raw_parts(slices_vec.as_ptr(), 2) };
        // let jagged = RawJaggedRef::new(slices, VecDequeSlicesIndexer, Some(vec_deque.len()));
        // let con_iter = ConIterCore::new(jagged, 0);

        // ConIterVecDequeRef {
        //     vec_deque,
        //     _slices_vec: slices_vec,
        //     con_iter,
        // }
        // todo!()
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

    fn into_seq_iter(mut self) -> Self::SequentialIter {
        let num_remaining = self.len();

        let (vec_deque, mut con_iter, slices) = (self.vec_deque, self.con_iter, self.slices);
        let slices = slices as *mut &'a [T];
        con_iter.clear();
        // let _vec_to_drop = unsafe { Vec::from_raw_parts(slices as *mut &'a [T], 0, 2) };
        for i in 0..2 {
            let p = unsafe { slices.add(i) };
            unsafe { p.drop_in_place() };
        }
        unsafe { drop_in_place(slices) };

        // let empty = ManuallyDrop::new(Vec::new());
        // let empty_ptr = empty.as_ptr();
        // let x = core::mem::replace(&mut self.slices, empty_ptr) as *mut &'a [T];
        // let _vec_to_drop = unsafe { Vec::from_raw_parts(x, 0, 2) };

        // self.slices_arr.clear();
        // let ptr = self.slices as *mut &'a [T];
        // unsafe { ptr.write(Default::default()) };
        // unsafe { ptr.add(1).write(Default::default()) };
        // let _vec_to_drop = unsafe { Vec::from_raw_parts(self.slices as *mut &'a [T], 2, 2) };
        // _vec_to_drop.leak();

        // let _ = ManuallyDrop::new(self.con_iter);

        // self.slices_arr[0] = Default::default();
        // self.slices_arr[1] = Default::default();
        // unsafe { self.slices_arr.set_len(0) };
        // let mut empty = vec![Default::default(), Default::default()];
        // core::mem::swap(&mut self._slices_arr, &mut empty);
        // drop(empty);
        // let _ = ManuallyDrop::new(self._slices_vec);
        let skip = vec_deque.len().saturating_sub(num_remaining);
        vec_deque.iter().skip(skip)
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
