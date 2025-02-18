use super::chunks_iter_vec::ChunksIterVec;
use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Element, Enumerated, Enumeration, IsEnumerated, IsNotEnumerated, Regular},
    implementations::ptr_utils::take,
};
use alloc::vec::Vec;
use core::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterVec<T, E = Regular>
where
    T: Send + Sync,
    E: Enumeration,
{
    ptr: *const T,
    vec_len: usize,
    vec_cap: usize,
    counter: AtomicUsize,
    phantom: PhantomData<E>,
}

unsafe impl<T: Send + Sync, E: Enumeration> Sync for ConIterVec<T, E> {}

unsafe impl<T: Send + Sync, E: Enumeration> Send for ConIterVec<T, E> {}

impl<T, E> Default for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T, E> Drop for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    fn drop(&mut self) {
        // # SAFETY
        // null ptr indicates that the data is already taken out of this iterator
        // by a consuming method such as `into_seq_iter`
        if !self.ptr.is_null() {
            unsafe { self.drop_elements_in_place(self.num_taken()..self.vec_len) };
            let _vec_to_drop = unsafe { Vec::from_raw_parts(self.ptr as *mut T, 0, self.vec_cap) };
        }
    }
}

impl<T, E> ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    pub(crate) fn new(mut vec: Vec<T>) -> Self {
        let (vec_len, vec_cap) = (vec.len(), vec.capacity());
        let ptr = vec.as_mut_ptr();
        let _ = ManuallyDrop::new(vec);
        Self {
            ptr,
            vec_len,
            vec_cap,
            counter: 0.into(),
            phantom: PhantomData,
        }
    }

    fn progress_and_get_begin_idx(&self, number_to_fetch: usize) -> Option<usize> {
        let begin_idx = self.counter.fetch_add(number_to_fetch, Ordering::Relaxed);
        match begin_idx < self.vec_len {
            true => Some(begin_idx),
            _ => None,
        }
    }

    pub(super) fn progress_and_get_chunk_ptrs(
        &self,
        chunk_size: usize,
    ) -> Option<(usize, *const T, *const T)> {
        self.progress_and_get_begin_idx(chunk_size)
            .map(|begin_idx| {
                let end_idx = (begin_idx + chunk_size).min(self.vec_len).max(begin_idx);
                let first = unsafe { self.ptr.add(begin_idx) } as *const T;
                let last = unsafe { self.ptr.add(end_idx - 1) } as *const T;
                (begin_idx, first, last)
            })
    }

    fn num_taken(&self) -> usize {
        self.counter.load(Ordering::Acquire).min(self.vec_len)
    }

    unsafe fn take_unchecked(&self, item_idx: usize) -> T {
        take(self.ptr.add(item_idx) as *mut T)
    }

    unsafe fn drop_elements_in_place(&self, range: Range<usize>) {
        for i in range {
            let p = self.ptr.add(i) as *mut T;
            p.drop_in_place();
        }
    }
}

impl<T, E> ConcurrentIter<E> for ConIterVec<T, E>
where
    T: Send + Sync,
    E: Enumeration,
{
    type Item = T;

    type SeqIter = alloc::vec::IntoIter<T>;

    type ChunkPuller<'i>
        = ChunksIterVec<'i, T, E>
    where
        Self: 'i;

    type Regular = ConIterVec<T, Regular>;

    type Enumerated = ConIterVec<T, Enumerated>;

    fn into_seq_iter(mut self) -> Self::SeqIter {
        let num_taken = self.num_taken();
        let ptr = self.ptr as *mut T;

        self.ptr = core::ptr::null_mut(); // to avoid double free on drop

        match num_taken {
            0 => {
                let vec = unsafe { Vec::from_raw_parts(ptr, self.vec_len, self.vec_cap) };
                vec.into_iter()
            }
            _ => {
                // TODO: ???
                let right_len = self.vec_len - num_taken;
                for i in 0..right_len {
                    let dst = unsafe { ptr.add(i) };
                    let src = unsafe { ptr.add(i + num_taken) };
                    unsafe { dst.swap(src) };
                }
                let vec = unsafe { Vec::from_raw_parts(ptr, right_len, self.vec_cap) };
                vec.into_iter()
            }
        }
    }

    fn enumerated(mut self) -> Self::Enumerated
    where
        E: IsNotEnumerated,
    {
        let ptr = self.ptr;
        self.ptr = core::ptr::null_mut();
        ConIterVec {
            ptr,
            vec_len: self.vec_len,
            vec_cap: self.vec_cap,
            counter: self.counter.load(Ordering::Acquire).into(),
            phantom: PhantomData,
        }
    }

    fn not_enumerated(mut self) -> Self::Regular
    where
        E: IsEnumerated,
    {
        let ptr = self.ptr;
        self.ptr = core::ptr::null_mut();
        ConIterVec {
            ptr,
            vec_len: self.vec_len,
            vec_cap: self.vec_cap,
            counter: self.counter.load(Ordering::Acquire).into(),
            phantom: PhantomData,
        }
    }

    fn skip_to_end(&self) {
        let num_taken_before = self.counter.fetch_max(self.vec_len, Ordering::Acquire);
        unsafe { self.drop_elements_in_place(num_taken_before..self.vec_len) };
    }

    fn next(&self) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<Self::Item>> {
        self.progress_and_get_begin_idx(1)
            .map(|idx| E::new_element(idx, unsafe { self.take_unchecked(idx) }))
    }

    fn chunks_iter(&self, chunk_size: usize) -> Self::ChunkPuller<'_> {
        Self::ChunkPuller::new(self, chunk_size)
    }
}
