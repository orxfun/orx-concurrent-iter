use crate::{concurrent_iter::ConcurrentIter, next::Next};
use core::{
    iter::Skip,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct ConIterSliceRef<'a, T> {
    slice: &'a [T],
    counter: AtomicUsize,
}

impl<'a, T> ConIterSliceRef<'a, T> {
    #[inline(always)]
    fn get(&self, item_idx: usize) -> Option<&'a T> {
        self.slice.get(item_idx)
    }
}

impl<'a, T> ConcurrentIter for ConIterSliceRef<'a, T> {
    type Item = &'a T;

    type SeqIter = Skip<core::slice::Iter<'a, T>>;

    fn into_seq_iter(self) -> Self::SeqIter {
        let current = self.counter.load(Ordering::Acquire);
        self.slice.iter().skip(current)
    }

    fn next<N: Next<Self::Item>>(&self) -> Option<N> {
        let idx = self.counter.fetch_add(1, Ordering::Acquire);
        self.get(idx).map(|value| N::new(idx, value))
    }

    fn skip_to_end(&self) {
        let _ = self.counter.fetch_max(self.slice.len(), Ordering::Acquire);
    }
}
