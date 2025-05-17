use crate::{
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::empty::con_iter::ConIterEmpty, pullers::ChunkPuller,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[test]
fn enumeration() {
    let iter = ConIterEmpty::<String>::new();

    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
}

#[test]
fn size_hint() {
    let iter = ConIterEmpty::<String>::new();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn size_hint_skip_to_end() {
    let iter = ConIterEmpty::<String>::new();

    iter.skip_to_end();
    assert_eq!(iter.len(), 0);
}

#[test_matrix([1, 2, 4])]
fn empty(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

#[test_matrix([1, 2, 4])]
fn next(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn next_with_idx(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn item_puller(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn item_puller_with_idx(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn chunk_puller(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn chunk_puller_with_idx(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn flattened_chunk_puller(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn flattened_chunk_puller_with_idx(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

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

    let mut expected: Vec<_> = (0..0).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([1, 2, 4])]
fn skip_to_end(nt: usize) {
    let iter = ConIterEmpty::<String>::new();

    let until = 0 / 2;

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    let con_num_spawned = &num_spawned;
    let con_bag = &bag;
    let con_iter = &iter;
    std::thread::scope(|s| {
        for t in 0..nt {
            s.spawn(move || {
                con_num_spawned.push(true);
                while con_num_spawned.len() < nt {} // allow all threads to be spawned

                match t % 2 {
                    0 => {
                        while let Some(num) = con_iter.next() {
                            match num.parse::<usize>().expect("") < until + 10 {
                                true => _ = con_bag.push(num),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                    _ => {
                        for num in con_iter.chunk_puller(7).flattened() {
                            match num.parse::<usize>().expect("") < until + 10 {
                                true => _ = con_bag.push(num),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..until).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test]
fn into_seq_iter() {
    let iter = ConIterEmpty::<String>::new();
    let iter = iter.into_seq_iter();
    let all: Vec<_> = iter.collect();

    let mut expected: Vec<_> = (0..0).map(|i| (i + 10).to_string()).collect();
    expected.sort();

    assert_eq!(all, expected);
}
