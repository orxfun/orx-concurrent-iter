use crate::concurrent_iter::ConcurrentIter;

pub trait ExactSizeConcurrentIter: ConcurrentIter {
    fn len(&self) -> usize;
}
