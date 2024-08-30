use crate::{ConIterOfSlice, ConcurrentIterable};

impl<const N: usize, T: Send + Sync> ConcurrentIterable for [T; N] {
    type Item<'i> = &'i T where Self: 'i;

    type ConIter<'i> = ConIterOfSlice<'i, T> where Self: 'i;

    fn con_iter(&self) -> Self::ConIter<'_> {
        Self::ConIter::new(self.as_slice())
    }
}
