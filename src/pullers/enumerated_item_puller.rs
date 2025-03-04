use crate::concurrent_iter::ConcurrentIter;

pub struct EnumeratedItemPuller<'a, I>
where
    I: ConcurrentIter,
{
    con_iter: &'a I,
}

impl<'i, I> From<&'i I> for EnumeratedItemPuller<'i, I>
where
    I: ConcurrentIter,
{
    fn from(con_iter: &'i I) -> Self {
        Self { con_iter }
    }
}

impl<I> Iterator for EnumeratedItemPuller<'_, I>
where
    I: ConcurrentIter,
{
    type Item = (usize, I::Item);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next_with_idx()
    }
}
