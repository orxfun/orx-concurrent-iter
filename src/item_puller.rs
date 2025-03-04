use crate::concurrent_iterator::ConcurrentIterator;

pub struct ItemPuller<'a, I>
where
    I: ConcurrentIterator,
{
    con_iter: &'a I,
}

impl<'i, I> From<&'i I> for ItemPuller<'i, I>
where
    I: ConcurrentIterator,
{
    fn from(con_iter: &'i I) -> Self {
        Self { con_iter }
    }
}

impl<I> Iterator for ItemPuller<'_, I>
where
    I: ConcurrentIterator,
{
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.con_iter.next()
    }
}
