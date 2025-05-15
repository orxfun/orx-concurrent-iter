use crate::{
    ChunkPuller, ConcurrentIter, ExactSizeConcurrentIter,
    implementations::jagged::{
        con_iter::ConIterJaggedRef, raw_jagged::RawJagged, tests::indexers::MatrixIndexer,
    },
};
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 11;
#[cfg(not(miri))]
const N: usize = 66;

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(
            ((i * n)..((i + 1) * n))
                .map(|x| (10 + x).to_string())
                .collect(),
        );
    }
    matrix
}

#[test]
fn enumeration() {
    let n = 2;
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    assert_eq!(iter.next(), Some(&10.to_string()));
    assert_eq!(iter.next_with_idx(), Some((1, &11.to_string())));
    assert_eq!(iter.next(), Some(&12.to_string()));
    assert_eq!(iter.next_with_idx(), Some((3, &13.to_string())));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
}

#[test]
fn size_hint() {
    let n = 5;
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let mut n = n * n;

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
fn size_hint_skip_to_end() {
    let n = 5;
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    for _ in 0..10 {
        let _ = iter.next();
    }
    let mut chunks_iter = iter.chunk_puller(7);
    let _ = chunks_iter.pull();

    assert_eq!(iter.len(), 8);

    iter.skip_to_end();
    assert_eq!(iter.len(), 0);
}

#[test_matrix([1, 2, 4])]
fn empty(nt: usize) {
    let n = 0;
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

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

#[test_matrix([0, 2, N], [1, 2, 4])]
fn next(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    _ = iter.size_hint();
                    bag.push(x.clone());
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn next_with_idx(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some((idx, x)) = iter.next_with_idx() {
                    _ = iter.size_hint();
                    bag.push((idx, x.clone()));
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn item_puller(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                for x in iter.item_puller() {
                    _ = iter.size_hint();
                    bag.push(x.clone());
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix( [0, 2, N], [1, 2, 4])]
fn item_puller_with_idx(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                for (idx, x) in iter.item_puller_with_idx() {
                    _ = iter.size_hint();
                    bag.push((idx, x.clone()));
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn chunk_puller(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

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
                        bag.push(x.clone());
                    }
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn chunk_puller_with_idx(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

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
                        bag.push((begin_idx + i, x.clone()));
                    }
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn flattened_chunk_puller(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                for x in iter.chunk_puller(7).flattened() {
                    bag.push(x.clone());
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn flattened_chunk_puller_with_idx(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                for (idx, x) in iter.chunk_puller(7).flattened_with_idx() {
                    bag.push((idx, x.clone()));
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i, (i + 10).to_string())).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 2, N], [1, 2, 4])]
fn skip_to_end(n: usize, nt: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let until = (n * n) / 2;

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
                                true => _ = con_bag.push(num.clone()),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                    _ => {
                        for num in con_iter.chunk_puller(7).flattened() {
                            match num.parse::<usize>().expect("") < until + 10 {
                                true => _ = con_bag.push(num.clone()),
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

#[test_matrix([0, 2, N], [1, 2, 4], [0, N * N / 2, N * N])]
fn into_seq_iter(n: usize, nt: usize, until: usize) {
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));
    let iter = ConIterJaggedRef::new(&jagged, 0);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    let con_num_spawned = &num_spawned;
    let con_bag = &bag;
    let con_iter = &iter;
    if until > 0 {
        std::thread::scope(|s| {
            for t in 0..nt {
                s.spawn(move || {
                    con_num_spawned.push(true);
                    while con_num_spawned.len() < nt {} // allow all threads to be spawned

                    match t % 2 {
                        0 => {
                            while let Some(num) = con_iter.next() {
                                con_bag.push(num.clone());
                                if num.parse::<usize>().expect("") >= until + 10 {
                                    break;
                                }
                            }
                        }
                        _ => {
                            let mut iter = con_iter.chunk_puller(7);
                            while let Some(chunk) = iter.pull() {
                                let mut do_break = false;
                                for num in chunk {
                                    con_bag.push(num.clone());
                                    if num.parse::<usize>().expect("") >= until + 10 {
                                        do_break = true;
                                    }
                                }
                                if do_break {
                                    break;
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    let iter = iter.into_seq_iter();
    let remaining: Vec<_> = iter.cloned().collect();
    let collected = bag.into_inner().to_vec();
    let mut all: Vec<_> = collected.into_iter().chain(remaining).collect();
    all.sort();

    let mut expected: Vec<_> = (0..(n * n)).map(|i| (i + 10).to_string()).collect();
    expected.sort();

    assert_eq!(all, expected);
}
