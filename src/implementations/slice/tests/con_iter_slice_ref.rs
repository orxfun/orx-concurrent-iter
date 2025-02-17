use crate::{
    concurrent_iter::ConcurrentIter, implementations::slice::con_iter_slice_ref::ConIterSliceRef,
};

#[test]
fn empty_slice() {
    let vec = Vec::<String>::new();
    let slice = vec.as_slice();
    let con_iter = ConIterSliceRef::<String>::new(slice);

    assert!(con_iter.next().is_none());
    assert!(con_iter.next().is_none());
    // assert!(con_iter.next::<&String>().is_none());
}
