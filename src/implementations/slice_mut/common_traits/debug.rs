use crate::{ExactSizeConcurrentIter, implementations::slice_mut::ConIterSliceMut};
use core::fmt::Debug;

impl<T> Debug for ConIterSliceMut<'_, T>
where
    T: Send + Sync,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let remaining = self.len();
        let num_taken = self.slice().len() - remaining;
        f.debug_struct("ConIterSliceMut")
            .field("initial_len", &self.slice().len())
            .field("num_taken", &num_taken)
            .field("remaining", &remaining)
            .finish()
    }
}
