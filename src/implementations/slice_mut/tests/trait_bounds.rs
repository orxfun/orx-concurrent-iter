fn into_con_iter<T: Send>(slice: &mut [T]) {
    use crate::IntoConcurrentIter;
    let _con_iter = slice.into_con_iter();
}

#[test]
fn slice_con_iter_trait_bounds() {
    into_con_iter(&mut [1, 2, 4]);
}
