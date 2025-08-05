use crate::{ExactSizeConcurrentIter, implementations::ConIterSlice};
use core::fmt::Debug;

impl<T> Debug for ConIterSlice<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let remaining = self.len();
        let num_taken = self.slice().len() - remaining;
        f.debug_struct("ConIterSlice")
            .field("initial_len", &self.slice().len())
            .field("num_taken", &num_taken)
            .field("remaining", &remaining)
            .finish()
    }
}
