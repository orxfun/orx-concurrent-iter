use crate::{ExactSizeConcurrentIter, implementations::ConIterVec};
use core::fmt::Debug;

impl<T> Debug for ConIterVec<T>
where
    T: Send + Sync,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let remaining = self.len();
        let num_taken = self.initial_len() - remaining;
        f.debug_struct("ConIterVec")
            .field("initial_len", &self.initial_len())
            .field("num_taken", &num_taken)
            .field("remaining", &remaining)
            .finish()
    }
}
