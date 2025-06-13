use crate::{ConcurrentDrainableOverSlice, implementations::vec_drain::con_iter::ConIterVecDrain};
use alloc::vec::Vec;
use core::ops::RangeBounds;

impl<T> ConcurrentDrainableOverSlice for Vec<T>
where
    T: Send + Sync,
{
    type Item = T;

    type DrainingIter<'a>
        = ConIterVecDrain<'a, T>
    where
        Self: 'a;

    fn con_drain<R>(&mut self, range: R) -> Self::DrainingIter<'_>
    where
        R: RangeBounds<usize>,
    {
        ConIterVecDrain::new(self, range)
    }
}
