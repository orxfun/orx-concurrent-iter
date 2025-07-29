use std::collections::VecDeque;

fn into_con_iter<T: Send + Sync>(vec: VecDeque<T>) {
    use crate::IntoConcurrentIter;
    let _con_iter = vec.into_con_iter();
}

fn concurrent_iterable<T: Send + Sync>(vec: VecDeque<T>) {
    use crate::ConcurrentIterable;
    let vec_ref = &vec;
    let _con_iter = vec_ref.con_iter();
}

fn concurrent_collection<T: Send + Sync>(vec: VecDeque<T>) {
    use crate::ConcurrentCollection;
    let _con_iter = vec.con_iter();
    let _con_iter = vec.as_concurrent_iterable();
}

#[test]
fn vec_con_iter_trait_bounds() {
    into_con_iter(VecDeque::<String>::new());
    concurrent_iterable(VecDeque::<String>::new());
    concurrent_collection(VecDeque::<String>::new());
}
