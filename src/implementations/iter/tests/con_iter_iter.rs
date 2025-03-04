use crate::{
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::ConIterOfIter, into_concurrent_iter::IntoConcurrentIter, pullers::ChunkPuller,
};
use core::ops::Range;
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 4735;

fn new_vec(n: usize, elem: impl Fn(usize) -> String) -> Vec<String> {
    let mut vec = Vec::with_capacity(n + 17);
    for i in 0..n {
        vec.push(elem(i));
    }
    vec
}

#[test]
fn enumeration() {
    let vec: Vec<_> = (0..6).collect();
    let iter = ConIterOfIter::new(vec.into_iter().filter(|x| *x < 99));

    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_with_idx(), Some((1, 1)));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_with_idx(), Some((3, 3)));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next_with_idx(), Some((5, 5)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
}

#[test]
fn size_hint() {
    let mut n = 25;
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = ConIterOfIter::new(vec.into_iter().map(|x| format!("{}!", x)));

    for _ in 0..10 {
        assert_eq!(iter.size_hint(), (n, Some(n)));
        let _ = iter.next();
        n -= 1;
    }

    let mut chunks_iter = iter.chunk_puller(7);

    assert_eq!(iter.size_hint(), (n, Some(n)));
    assert_eq!(iter.len(), n);
    let _ = chunks_iter.pull();
    n -= 7;

    assert_eq!(iter.size_hint(), (n, Some(n)));
    assert_eq!(iter.len(), n);
    let _ = chunks_iter.pull();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let _ = chunks_iter.pull();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let _ = chunks_iter.pull();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let _ = iter.next();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn size_hint_unknown() {
    let mut n = 25;
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = ConIterOfIter::new(vec.into_iter().filter(|x| x.starts_with("1")));

    for _ in 0..10 {
        assert_eq!(iter.size_hint(), (0, Some(n)));
        assert_eq!(iter.try_get_len(), None);
        let _ = iter.next();
        n -= 1;
    }

    let mut chunks_iter = iter.chunk_puller(7);

    assert_eq!(iter.size_hint(), (0, Some(n)));
    assert_eq!(iter.try_get_len(), None);
    let _ = chunks_iter.pull();
    n -= 7;

    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.try_get_len(), Some(0));

    let chunk = chunks_iter.pull();
    assert!(chunk.is_none());
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn size_hint_skip_to_end() {
    let n = 25;
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = ConIterOfIter::new(vec.into_iter().map(|x| format!("{}!", x)));

    for _ in 0..10 {
        let _ = iter.next();
    }
    let mut chunks_iter = iter.chunk_puller(7);
    let _ = chunks_iter.pull();

    assert_eq!(iter.len(), 8);

    iter.skip_to_end();
    assert_eq!(iter.len(), 0);
}
