fn into_con_iter<T: Sync>(slice: &[T]) {
    use crate::IntoConcurrentIter;
    let _con_iter = slice.into_con_iter();
}

fn concurrent_iterable<T: Sync>(slice: &[T]) {
    use crate::ConcurrentIterable;
    let _con_iter = slice.con_iter();
}

#[test]
fn slice_con_iter_trait_bounds() {
    into_con_iter(&[1, 2, 4]);
    concurrent_iterable(&[1, 2, 4]);
}
