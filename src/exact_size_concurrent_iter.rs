use crate::concurrent_iter::ConcurrentIter;

pub trait ExactSizeConcurrentIter: ConcurrentIter {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
