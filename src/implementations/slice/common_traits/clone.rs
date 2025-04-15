use crate::{ExactSizeConcurrentIter, implementations::ConIterSlice};

impl<T> Clone for ConIterSlice<'_, T>
where
    T: Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        let remaining = self.len();
        let num_taken = self.slice().len() - remaining;
        let slice = &self.slice()[num_taken..];
        Self::new(slice)
    }
}
