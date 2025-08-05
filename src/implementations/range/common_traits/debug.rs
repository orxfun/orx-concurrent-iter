use crate::{ExactSizeConcurrentIter, implementations::ConIterRange};
use core::{fmt::Debug, ops::Range};

impl<T> Debug for ConIterRange<T>
where
    T: From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let remaining = self.len();
        let num_taken = self.initial_len() - remaining;
        f.debug_struct("ConIterRange")
            .field("initial_len", &self.initial_len())
            .field("num_taken", &num_taken)
            .field("remaining", &remaining)
            .finish()
    }
}
