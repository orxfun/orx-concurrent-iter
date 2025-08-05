use crate::{ExactSizeConcurrentIter, implementations::ConIterRange};
use core::ops::Range;

impl<T> Clone for ConIterRange<T>
where
    T: Send + From<usize> + Into<usize>,
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    fn clone(&self) -> Self {
        let begin = self.begin();
        let remaining = self.len();
        let num_taken = self.initial_len() - remaining;
        let first = begin + num_taken;
        let last = first + remaining;
        let range = T::from(first)..T::from(last);
        ConIterRange::new(range)
    }
}
