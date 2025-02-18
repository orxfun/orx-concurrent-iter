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

    let mut puller = con_iter.in_chunks(5);
    assert!(puller.pull().is_none());

    let mut iter = con_iter.in_chunks(5).flatten();
    assert!(iter.next().is_none());
}

#[test_matrix([Regular, Enumerated])]
fn next<K: NextKind>(_: K) {
    let n = 123;
    let vec: Vec<_> = (0..n).map(|x| x + 10).collect();
    let slice = vec.as_slice();
    let con_iter = ConIterSliceRef::<_, K>::new(slice);
    for i in 0..n {
        let x = i + 10;
        let next = con_iter.next().unwrap();
        assert!(K::eq_next(next, K::new_next(i, &x)));
    }
}

#[test_matrix([Regular, Enumerated])]
fn in_chunks<K: NextKind>(_: K) {
    let n = 123;
    let vec: Vec<_> = (0..n).map(|x| x + 10).collect();
    let slice = vec.as_slice();
    let con_iter = ConIterSliceRef::<_, K>::new(slice);
    let mut puller = con_iter.in_chunks(5);
    let mut i = 0;
    while let Some(x) = puller.pull() {
        let (begin_idx, iter) = K::destruct_next(x);
        assert!(K::eq_begin_idx(begin_idx, i));

        match i {
            120 => assert_eq!(iter.len(), 3),
            _ => assert_eq!(iter.len(), 5),
        };
        for x in iter {
            assert_eq!(*x, i + 10);
            i += 1;
        }
    }
}
