use crate::implementations::array_utils::ArrayIntoSeqIter;
use core::{iter::FusedIterator, marker::PhantomData};

pub struct ArrayChunkSeqIter<'i, T> {
    iter: ArrayIntoSeqIter<T, ()>,
    parent_iter_lifetime: PhantomData<&'i ()>,
}

impl<T> ArrayChunkSeqIter<'_, T> {
    pub(crate) fn new(first: *const T, last: *const T) -> Self {
        Self {
            iter: ArrayIntoSeqIter::new(first, last, None, ()),
            parent_iter_lifetime: PhantomData,
        }
    }
}

impl<T> Default for ArrayChunkSeqIter<'_, T> {
    fn default() -> Self {
        Self::new(core::ptr::null(), core::ptr::null())
    }
}

impl<T> Iterator for ArrayChunkSeqIter<'_, T> {
    type Item = T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for ArrayChunkSeqIter<'_, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for ArrayChunkSeqIter<'_, T> {}
