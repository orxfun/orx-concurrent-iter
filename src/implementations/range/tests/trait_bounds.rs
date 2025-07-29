use core::ops::Range;

fn into_con_iter<T: Send + Sync + From<usize> + Into<usize>>(range: Range<T>)
where
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    use crate::IntoConcurrentIter;
    let _con_iter = range.into_con_iter();
}

fn concurrent_iterable<T: Send + Sync + From<usize> + Into<usize>>(range: Range<T>)
where
    Range<T>: Default + Clone + ExactSizeIterator<Item = T>,
{
    use crate::ConcurrentIterable;
    let _con_iter = range.con_iter();
}

#[test]
fn slice_con_iter_trait_bounds() {
    into_con_iter(0..1);
    concurrent_iterable(0..1);
}
