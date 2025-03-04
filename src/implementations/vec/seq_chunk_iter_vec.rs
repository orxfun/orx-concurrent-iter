use super::vec_into_seq_iter::VecIntoSeqIter;
use core::{iter::FusedIterator, marker::PhantomData};

pub struct SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    iter: VecIntoSeqIter<T>,
    phantom: PhantomData<&'i ()>,
}

impl<T> SeqChunksIterVec<'_, T>
where
    T: Send + Sync,
{
    pub(super) fn new(completed: bool, first: *const T, last: *const T) -> Self {
        Self {
            iter: VecIntoSeqIter::new(completed, first, last, first, None),
            phantom: PhantomData,
        }
    }
}

impl<T> Default for SeqChunksIterVec<'_, T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        let p: *const T = core::ptr::null();
        Self::new(true, p, p)
    }
}

impl<T> Iterator for SeqChunksIterVec<'_, T>
where
    T: Send + Sync,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for SeqChunksIterVec<'_, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T> FusedIterator for SeqChunksIterVec<'_, T> where T: Send + Sync {}
