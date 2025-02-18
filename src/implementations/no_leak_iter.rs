/// An iterator which guarantees that all items are yielded on drop,
/// even if the owner did not consume all the elements.
///
/// This guarantees that all elements will be yield and dropped,
/// which could otherwise leak.
pub struct NoLeakIter<I: Iterator>(I);

impl<I: Iterator> From<I> for NoLeakIter<I> {
    fn from(iter: I) -> Self {
        Self(iter)
    }
}

impl<I: Iterator> Drop for NoLeakIter<I> {
    fn drop(&mut self) {
        for _ in self.0.by_ref() {}
    }
}

impl<I: Iterator> Iterator for NoLeakIter<I> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for NoLeakIter<I> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
}
