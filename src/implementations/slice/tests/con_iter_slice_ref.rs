use crate::{
    chunk_puller::ChunkPuller,
    concurrent_iter::ConcurrentIter,
    implementations::slice::con_iter_slice_ref::ConIterSliceRef,
    next::{Enumerated, NextKind, Regular},
};
use test_case::test_matrix;

#[test_matrix([Regular, Enumerated])]
fn empty_slice<K: NextKind>(_: K) {
    let vec = Vec::<String>::new();
    let slice = vec.as_slice();
    let con_iter = ConIterSliceRef::<String, K>::new(slice);

    assert!(con_iter.next().is_none());
    assert!(con_iter.next().is_none());

    assert!(con_iter.next_chunk(4).is_none());
    assert!(con_iter.next_chunk(4).is_none());

    let mut buf = con_iter.in_chunks(5);
    assert!(buf.pull().is_none());
}
