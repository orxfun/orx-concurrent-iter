use super::vec_into_seq_iter::VecIntoSeqIter;
use core::{iter::FusedIterator, marker::PhantomData};

pub struct SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    iter: VecIntoSeqIter<T>,
    phantom: PhantomData<&'i ()>,
}

impl<'i, T> SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    pub(super) fn new(first: *const T, last: *const T) -> Self {
        Self {
            iter: VecIntoSeqIter::new(first, last, first, None),
            phantom: PhantomData,
        }
    }
}

impl<'i, T> Default for SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        let p: *const T = core::ptr::null();
        Self::new(p.clone(), p)
    }
}

impl<'i, T> Iterator for SeqChunksIterVec<'i, T>
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

impl<'i, T> ExactSizeIterator for SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'i, T> FusedIterator for SeqChunksIterVec<'i, T> where T: Send + Sync {}
