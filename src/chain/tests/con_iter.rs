use crate::{
    IntoConcurrentIter, IterIntoConcurrentIter, chain::con_iter_known_len_i::ChainKnownLenI,
    chain::con_iter_unknown_len_i::ChainUnknownLenI, concurrent_iter::ConcurrentIter,
    exact_size_concurrent_iter::ExactSizeConcurrentIter, pullers::ChunkPuller,
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
    test(ChainKnownLenI::new(
        (0..3).collect::<Vec<_>>().into_con_iter(),
        (3..6).collect::<Vec<_>>().into_con_iter(),
        3,
    ));
    test(ChainUnknownLenI::new(
        (0..3)
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|x| x < &100)
            .iter_into_con_iter(),
        (3..6).collect::<Vec<_>>().into_con_iter(),
    ));

    fn test(iter: impl ConcurrentIter<Item = usize>) {
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
}

#[test]
fn size_hint() {
    let v1 = || new_vec(12, |x| (x + 10).to_string());
    let v2 = || new_vec(13, |x| (x + 10).to_string());

    test_k_k(ChainKnownLenI::new(
        v1().into_con_iter(),
        v2().into_con_iter(),
        12,
    ));
    test_u_k(ChainUnknownLenI::new(
        v1().into_iter()
            .filter(|x| !x.starts_with('x'))
            .iter_into_con_iter(),
        v2().into_con_iter(),
    ));
    test_k_u(ChainKnownLenI::new(
        v1().into_con_iter(),
        v2().into_iter()
            .filter(|x| !x.starts_with('x'))
            .iter_into_con_iter(),
        12,
    ));
    test_u_u(ChainUnknownLenI::new(
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

#[test]
fn size_hint_skip_to_end() {
    let v1 = || new_vec(12, |x| (x + 10).to_string());
    let v2 = || new_vec(13, |x| (x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, chunk: usize) {
        for _ in 0..10 {
            let _ = iter.next();
        }
        let mut chunks_iter = iter.chunk_puller(chunk);
        let _ = chunks_iter.pull();

        iter.skip_to_end();

        assert_eq!(iter.next(), None);
        assert!(chunks_iter.pull().is_none());
    }

    for chunk in [4, 7, 12, 13, 20, 30] {
        test(
            ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), 12),
            chunk,
        );
        test(
            ChainKnownLenI::new(
                v1().into_con_iter(),
                v2().into_iter()
                    .filter(|x| !x.starts_with('x'))
                    .iter_into_con_iter(),
                12,
            ),
            chunk,
        );
        test(
            ChainUnknownLenI::new(
                v1().into_iter()
                    .filter(|x| !x.starts_with('x'))
                    .iter_into_con_iter(),
                v2().into_con_iter(),
            ),
            chunk,
        );
        test(
            ChainUnknownLenI::new(
                v1().into_iter()
                    .filter(|x| !x.starts_with('x'))
                    .iter_into_con_iter(),
                v2().into_iter()
                    .filter(|x| !x.starts_with('x'))
                    .iter_into_con_iter(),
            ),
            chunk,
        );
    }
}

#[test_matrix([1, 2, 4])]
fn empty(nt: usize) {
    let v1 = || new_vec(0, |x| (x + 10).to_string());
    let v2 = || new_vec(0, |x| (x + 10).to_string());
    fn test(iter: impl ConcurrentIter, nt: usize) {
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    assert!(iter.next().is_none());
                    assert!(iter.next().is_none());

                    let mut puller = iter.chunk_puller(5);
                    assert!(puller.pull().is_none());
                    assert!(puller.pull().is_none());

                    let mut iter = iter.chunk_puller(5).flattened();
                    assert!(iter.next().is_none());
                    assert!(iter.next().is_none());
                });
            }
        });
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), 0),
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| x.starts_with('x'))
                .iter_into_con_iter(),
            0,
        ),
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn next(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    while let Some(x) = iter.next() {
                        _ = iter.size_hint();
                        bag.push(x);
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn next_with_idx(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    while let Some(x) = iter.next_with_idx() {
                        _ = iter.size_hint();
                        bag.push(x);
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i, (i + 10).to_string())).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn item_puller(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    for x in iter.item_puller() {
                        _ = iter.size_hint();
                        bag.push(x);
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix( [0, 1, N], [1, 2, 4])]
fn item_puller_with_idx(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    for x in iter.item_puller_with_idx() {
                        _ = iter.size_hint();
                        bag.push(x);
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i, (i + 10).to_string())).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn chunk_puller(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    let mut puller = iter.chunk_puller(7);

                    while let Some(chunk) = puller.pull() {
                        assert!(chunk.len() <= 7);
                        for x in chunk {
                            bag.push(x);
                        }
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn chunk_puller_with_idx(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    let mut puller = iter.chunk_puller(7);

                    while let Some((begin_idx, chunk)) = puller.pull_with_idx() {
                        assert!(chunk.len() <= 7);
                        for (i, x) in chunk.enumerate() {
                            bag.push((begin_idx + i, x));
                        }
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i, (i + 10).to_string())).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn flattened_chunk_puller(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    for x in iter.chunk_puller(7).flattened() {
                        bag.push(x);
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn flattened_chunk_puller_with_idx(n: usize, nt: usize) {
    let v1 = || new_vec(n / 3, |x| (x + 10).to_string());
    let v2 = || new_vec(n - n / 3, |x| (n / 3 + x + 10).to_string());
    fn test(iter: impl ConcurrentIter<Item = String>, n: usize, nt: usize) {
        let bag = ConcurrentBag::new();
        let num_spawned = ConcurrentBag::new();
        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    num_spawned.push(true);
                    while num_spawned.len() < nt {} // allow all threads to be spawned

                    for x in iter.chunk_puller(7).flattened_with_idx() {
                        bag.push(x);
                    }
                });
            }
        });

        let mut expected: Vec<_> = (0..n).map(|i| (i, (i + 10).to_string())).collect();
        expected.sort();
        let mut collected = bag.into_inner().to_vec();
        collected.sort();

        assert_eq!(expected, collected);
    }
    test(
        ChainKnownLenI::new(v1().into_con_iter(), v2().into_con_iter(), n / 3),
        n,
        nt,
    );
    test(
        ChainKnownLenI::new(
            v1().into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            n / 3,
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_con_iter(),
        ),
        n,
        nt,
    );
    test(
        ChainUnknownLenI::new(
            v1().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
            v2().into_iter()
                .filter(|x| !x.starts_with('x'))
                .iter_into_con_iter(),
        ),
        n,
        nt,
    );
}
