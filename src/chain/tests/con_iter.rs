use crate::{
    IntoConcurrentIter, IterIntoConcurrentIter, chain::con_iter::Chain,
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    pullers::ChunkPuller,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
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
    let v1: Vec<_> = (0..3).collect();
    let i1 = v1.into_con_iter();

    let v2: Vec<_> = (3..6).collect();
    let s2 = v2.as_slice();
    let i2 = s2.into_con_iter().copied();

    let iter = Chain::new(i1, i2);

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
    let v1 = || new_vec(12, |x| (x + 10).to_string());
    let v2 = || new_vec(13, |x| (x + 10).to_string());

    test_k_k(Chain::new(v1().into_con_iter(), v2().into_con_iter()));
    test_u_k(Chain::new(
        v1().into_iter()
            .filter(|x| !x.starts_with('x'))
            .iter_into_con_iter(),
        v2().into_con_iter(),
    ));
    test_k_u(Chain::new(
        v1().into_con_iter(),
        v2().into_iter()
            .filter(|x| !x.starts_with('x'))
            .iter_into_con_iter(),
    ));
    test_u_u(Chain::new(
        v1().into_iter()
            .filter(|x| !x.starts_with('x'))
            .iter_into_con_iter(),
        v2().into_iter()
            .filter(|x| !x.starts_with('x'))
            .iter_into_con_iter(),
    ));

    fn test_u_k(iter: impl ConcurrentIter<Item = String>) {
        let mut n = 25;
        for _ in 0..10 {
            assert_eq!(iter.size_hint(), (13, Some(n)));
            let _ = iter.next();
            n -= 1;
        }

        let mut chunks_iter = iter.chunk_puller(7);

        {
            assert_eq!(iter.size_hint(), (13, Some(15)));
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 2);
        }

        {
            assert_eq!(iter.size_hint(), (13, Some(13)));
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 7);
            assert_eq!(iter.size_hint(), (6, Some(6)));
        }

        {
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 6);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        {
            let _ = chunks_iter.pull();
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        let _ = iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    fn test_k_u(iter: impl ConcurrentIter<Item = String>) {
        for i in 0..10 {
            assert_eq!(iter.size_hint(), (12 - i, Some(25 - i)));
            let _ = iter.next();
        }

        let mut chunks_iter = iter.chunk_puller(7);

        {
            assert_eq!(iter.size_hint(), (2, Some(15)));
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 2);
        }

        {
            assert_eq!(iter.size_hint(), (0, Some(13)));
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 7);
            assert_eq!(iter.size_hint(), (0, Some(6)));
        }

        {
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 6);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        {
            let _ = chunks_iter.pull();
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        let _ = iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    fn test_k_k(iter: impl ExactSizeConcurrentIter<Item = String>) {
        let mut n = 25;
        for _ in 0..10 {
            assert_eq!(iter.size_hint(), (n, Some(n)));
            let _ = iter.next();
            n -= 1;
        }

        let mut chunks_iter = iter.chunk_puller(7);

        {
            assert_eq!(iter.size_hint(), (15, Some(15)));
            assert_eq!(iter.len(), 15);
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 2);
        }

        {
            assert_eq!(iter.size_hint(), (13, Some(13)));
            assert_eq!(iter.len(), 13);
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 7);
            assert_eq!(iter.size_hint(), (6, Some(6)));
        }
        {
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 6);
            assert_eq!(iter.len(), 0);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }
        {
            let _ = chunks_iter.pull();
            assert_eq!(iter.len(), 0);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        let _ = iter.next();
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    fn test_u_u(iter: impl ConcurrentIter<Item = String>) {
        for i in 0..10 {
            assert_eq!(iter.size_hint(), (0, Some(25 - i)));
            let _ = iter.next();
        }

        let mut chunks_iter = iter.chunk_puller(7);

        {
            assert_eq!(iter.size_hint(), (0, Some(15)));
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 2);
        }

        {
            assert_eq!(iter.size_hint(), (0, Some(13)));
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 7);
            assert_eq!(iter.size_hint(), (0, Some(6)));
        }

        {
            let c = chunks_iter.pull().unwrap();
            assert_eq!(c.len(), 6);
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        {
            let _ = chunks_iter.pull();
            assert_eq!(iter.size_hint(), (0, Some(0)));
        }

        let _ = iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
