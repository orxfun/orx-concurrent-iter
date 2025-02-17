use crate::iter::con_iter_x::ConcurrentIterX;

/// A regular `Iterator` crated from the [`ConcurrentIter::values`] method.
///
/// Its `next` method does nothing but call [`ConcurrentIter::next`] method.
/// This iterator is a wrapper to allow using the concurrent iterator in a `for` loop directly.
pub struct ConIterValuesX<'a, C>
where
    C: ConcurrentIterX,
{
    con_iter: &'a C,
}

impl<'a, C: ConcurrentIterX> From<&'a C> for ConIterValuesX<'a, C> {
    fn from(con_iter: &'a C) -> Self {
        Self { con_iter }
    }
}

impl<C> Iterator for ConIterValuesX<'_, C>
where
    C: ConcurrentIterX,
{
    type Item = C::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next()
    }
}
