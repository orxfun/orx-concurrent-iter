use crate::concurrent_iter::ConcurrentIter;

pub struct ItemPuller<'a, I>
where
    I: ConcurrentIter,
{
    con_iter: &'a I,
}

impl<'i, I> From<&'i I> for ItemPuller<'i, I>
where
    I: ConcurrentIter,
{
    fn from(con_iter: &'i I) -> Self {
        Self { con_iter }
    }
}

impl<I> Iterator for ItemPuller<'_, I>
where
    I: ConcurrentIter,
{
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next()
    }
}
