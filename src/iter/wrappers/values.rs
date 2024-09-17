use crate::ConcurrentIter;

/// A regular `Iterator` crated from the `values` method.
///
/// Its `next` method does nothing but call `next` method.
/// This iterator is a wrapper to allow using the concurrent iterator in a `for` loop directly.
pub struct ConIterValues<'a, C>
where
    C: ConcurrentIter,
{
    con_iter: &'a C,
}

impl<'a, C: ConcurrentIter> From<&'a C> for ConIterValues<'a, C> {
    fn from(con_iter: &'a C) -> Self {
        Self { con_iter }
    }
}

impl<'a, C> Iterator for ConIterValues<'a, C>
where
    C: ConcurrentIter,
{
    type Item = C::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next()
    }
}
