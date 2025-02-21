use crate::{
    chunk_puller::ChunkPuller, concurrent_iter::ConcurrentIter,
    implementations::iter::con_iter_of_iter::ConIterOfIter,
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

#[test_matrix([1, 2, 4])]
fn empty_iter(nt: usize) {
    let vecs = [Vec::<String>::new(), new_vec(56, |x| (x + 10).to_string())];
    for vec in vecs {
        let iter = vec.iter().filter(|x| x.as_str() == "abc");
        let iter = ConIterOfIter::<_, &String>::new(iter);

        std::thread::scope(|s| {
            for _ in 0..nt {
                s.spawn(|| {
                    assert!(iter.next().is_none());
                    assert!(iter.next().is_none());

                    let mut puller = iter.chunks_iter(5);
                    assert!(puller.pull().is_none());
                    assert!(puller.pull().is_none());

                    let mut iter = iter.chunks_iter(5).flattened();
                    assert!(iter.next().is_none());
                    assert!(iter.next().is_none());
                });
            }
        });
    }
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn next(n: usize, nt: usize) {
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = vec.iter().filter(|x| x.as_str() != "abc");
    let iter = ConIterOfIter::<_, &String>::new(iter);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned
                while let Some(x) = iter.next() {
                    bag.push(x);
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..n).map(|i| &vec[i]).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn chunks_iter(n: usize, nt: usize) {
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = vec.iter().filter(|x| x.as_str() != "abc");
    let iter = ConIterOfIter::<_, &String>::new(iter);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let mut chunks_iter = iter.chunks_iter(7);
                while let Some(chunk) = chunks_iter.pull() {
                    assert!(chunk.len() <= 7);
                    for x in chunk {
                        bag.push(x);
                    }
                }
            });
        }
    });

    let mut expected = vec![];
    for i in 0..n {
        expected.push(&vec[i]);
    }
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn chunks_iter_flattened(n: usize, nt: usize) {
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = vec.iter().filter(|x| x.as_str() != "abc");
    let iter = ConIterOfIter::<_, &String>::new(iter);

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                let chunks_iter = iter.chunks_iter(7).flattened();

                for x in chunks_iter {
                    bag.push(x);
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..n).map(|i| &vec[i]).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn skip_to_end(n: usize, nt: usize) {
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = vec.iter().filter(|x| x.as_str() != "abc");
    let iter = ConIterOfIter::<_, &String>::new(iter);
    let until = n / 2;

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
                        while let Some(x) = con_iter.next() {
                            let num: usize = x.parse().unwrap();
                            match num < until + 10 {
                                true => _ = con_bag.push(x),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                    _ => {
                        for x in con_iter.chunks_iter(7).flattened() {
                            let num: usize = x.parse().unwrap();
                            match num < until + 10 {
                                true => _ = con_bag.push(x),
                                false => con_iter.skip_to_end(),
                            }
                        }
                    }
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..until).map(|i| &vec[i]).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test_matrix([0, 1, N], [1, 2, 4], [0, N / 2, N])]
fn into_seq_iter(n: usize, nt: usize, until: usize) {
    let vec = new_vec(n, |x| (x + 10).to_string());
    let iter = vec.iter().filter(|x| x.as_str() != "abc");
    let iter = ConIterOfIter::<_, &String>::new(iter);

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
                            while let Some(x) = con_iter.next() {
                                let num: usize = x.parse().unwrap();
                                con_bag.push(num);
                                if num >= until + 10 {
                                    break;
                                }
                            }
                        }
                        _ => {
                            let mut iter = con_iter.chunks_iter(7);
                            while let Some(chunk) = iter.pull() {
                                let mut do_break = false;
                                for x in chunk {
                                    let num: usize = x.parse().unwrap();
                                    con_bag.push(num);
                                    if num >= until + 10 {
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
    let remaining: Vec<usize> = iter.map(|x| x.parse().unwrap()).collect();
    let collected = bag.into_inner().to_vec();
    let mut all: Vec<_> = collected.into_iter().chain(remaining.into_iter()).collect();
    all.sort();

    let expected: Vec<_> = (0..n).map(|i| i + 10).collect();

    assert_eq!(all, expected);
}
