use crate::ConcurrentIter;

/// A regular `Iterator` crated from the [`ConcurrentIter::ids_and_values`] method.
///
/// Its `next` method does nothing but call [`ConcurrentIter::next_id_and_value`] method.
/// This iterator is a wrapper to allow using the concurrent iterator in a `for` loop directly.
pub struct ConIterIdsAndValues<'a, C>
where
    C: ConcurrentIter,
{
    con_iter: &'a C,
}

impl<'a, C: ConcurrentIter> From<&'a C> for ConIterIdsAndValues<'a, C> {
    fn from(con_iter: &'a C) -> Self {
        Self { con_iter }
    }
}

impl<C> Iterator for ConIterIdsAndValues<'_, C>
where
    C: ConcurrentIter,
{
    type Item = (usize, C::Item);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next_id_and_value().map(|x| (x.idx, x.value))
    }
}
