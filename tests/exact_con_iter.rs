use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::*;
use std::{fmt::Debug, ops::Add};

#[cfg(not(miri))]
const LEN: usize = 8564;
#[cfg(miri)]
const LEN: usize = 77;

fn validate_exact_iter<Fun, I>(get_iter: Fun, expected_len: usize)
where
    I: ExactSizeConcurrentIter,
    I::Item: Debug + Add<usize, Output = usize>,
    Fun: Fn() -> I,
{
    let iter = get_iter();
    assert_eq!(iter.try_get_len(), Some(expected_len));
    assert_eq!(iter.len(), expected_len);

    for (idx, i) in iter.enumerate().item_puller() {
        assert_eq!(i + 0, idx);
    }

    let iter = get_iter();
    for (idx, i) in iter.item_puller_with_idx() {
        assert_eq!(i + 0, idx);
    }

    let iter = get_iter();
    let mut puller = iter.chunk_puller(7);
    while let Some((begin_idx, chunk)) = puller.pull_with_idx() {
        for (idx, i) in chunk.enumerate() {
            let idx = begin_idx + idx;
            assert_eq!(i + 0, idx);
        }
    }
}

fn validate_exact_iter_concurrently<Fun, I>(get_iter: Fun)
where
    I: ConcurrentIter + Sync,
    I::Item: Sync,
    I::Item: Debug + Add<usize, Output = usize> + PartialOrd + Ord,
    Fun: Fn() -> I,
{
    let num_threads = 4;

    let bag = ConcurrentBag::new();
    let con_bag = &bag;
    let iter = get_iter();
    let con_iter = &iter;
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                for value in con_iter.item_puller() {
                    con_bag.push(value);
                }
            });
        }
    });
    let mut vec = bag.into_inner().to_vec();
    vec.sort();
    for (i, value) in vec.into_iter().enumerate() {
        assert_eq!(value + 0, i);
    }

    let iter = get_iter();
    let con_iter = &iter;
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                for (idx, value) in con_iter.item_puller_with_idx() {
                    assert_eq!(idx, value + 0);
                }
            });
        }
    });

    let bag = ConcurrentBag::new();
    let con_bag = &bag;
    let iter = get_iter();
    let con_iter = &iter;
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                let mut puller = con_iter.chunk_puller(7);
                while let Some(chunk) = puller.pull() {
                    for x in chunk {
                        con_bag.push(x);
                    }
                }
            });
        }
    });
    let mut vec = bag.into_inner().to_vec();
    vec.sort();
    for (i, value) in vec.into_iter().enumerate() {
        assert_eq!(value + 0, i);
    }

    let iter = get_iter();
    let con_iter = &iter;
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                let mut puller = con_iter.chunk_puller(7);
                while let Some((begin_idx, chunk)) = puller.pull_with_idx() {
                    let len = chunk.len();
                    assert!(len > 0);
                    for (i, value) in chunk.enumerate() {
                        assert_eq!(begin_idx + i, value + 0);
                    }
                }
            });
        }
    });
}

#[test]
fn exact_range() {
    let range = 0..LEN;
    validate_exact_iter(|| range.clone().con_iter(), range.len());
    validate_exact_iter_concurrently(|| range.clone().con_iter());
}

#[test]
fn exact_vec() {
    let vec: Vec<_> = (0..LEN).collect();
    validate_exact_iter(|| vec.clone().into_con_iter(), vec.len());
    validate_exact_iter_concurrently(|| vec.clone().into_con_iter());
}

#[test]
fn exact_slice() {
    let vec: Vec<_> = (0..LEN).collect();
    validate_exact_iter(|| vec.con_iter(), vec.len());
    validate_exact_iter(|| vec.as_slice().into_con_iter(), vec.len());
    validate_exact_iter_concurrently(|| vec.as_slice().into_con_iter());
}

#[test]
fn exact_array() {
    let mut array = [0usize; LEN];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    validate_exact_iter(|| array.into_con_iter(), array.len());
    validate_exact_iter(|| array.into_con_iter(), array.len());
    validate_exact_iter_concurrently(|| array.into_con_iter());
}
