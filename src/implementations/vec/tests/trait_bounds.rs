fn into_con_iter<T: Send + Sync>(vec: Vec<T>) {
    use crate::IntoConcurrentIter;
    let _con_iter = vec.into_con_iter();
}

fn concurrent_iterable<T: Send + Sync>(vec: Vec<T>) {
    use crate::ConcurrentIterable;
    let vec_ref = &vec;
    let _con_iter = vec_ref.con_iter();
}

fn concurrent_collection<T: Send + Sync>(vec: Vec<T>) {
    use crate::ConcurrentCollection;
    let _con_iter = vec.con_iter();
    let _con_iter = vec.as_concurrent_iterable();
}

fn concurrent_collection_mut<T: Send + Sync>(mut vec: Vec<T>) {
    use crate::ConcurrentCollectionMut;
    let _con_iter = vec.con_iter_mut();
}

#[test]
fn vec_con_iter_trait_bounds() {
    into_con_iter(Vec::<String>::new());
    concurrent_iterable(Vec::<String>::new());
    concurrent_collection(Vec::<String>::new());
    concurrent_collection_mut(Vec::<String>::new());
}
