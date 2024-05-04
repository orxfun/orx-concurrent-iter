use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::*;
use std::{fmt::Debug, ops::Add};

fn validate_exact_iter<Fun, I>(get_iter: Fun, expected_len: usize)
where
    I: ExactSizeConcurrentIter,
    I::Item: Debug + Add<usize, Output = usize>,
    Fun: Fn() -> I,
{
    let iter = get_iter();
    assert_eq!(iter.len(), expected_len);

    for (idx, i) in iter.values().enumerate() {
        assert_eq!(i + 0, idx);
    }

    let iter = get_iter();
    for (idx, i) in iter.ids_and_values() {
        assert_eq!(i + 0, idx);
    }

    let iter = get_iter();
    while let Some(chunk) = iter.next_chunk(7) {
        for (idx, i) in chunk.values.enumerate() {
            let idx = chunk.begin_idx + idx;
            assert_eq!(i + 0, idx);
        }
    }
}

fn validate_exact_iter_concurrently<Fun, I>(get_iter: Fun)
where
    I: ExactSizeConcurrentIter,
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
                for value in con_iter.values() {
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
                for (idx, value) in con_iter.ids_and_values() {
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
                while let Some(chunk) = con_iter.next_chunk(7) {
                    for x in chunk.values {
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
                while let Some(chunk) = con_iter.next_chunk(7) {
                    let len = chunk.values.len();
                    assert!(len > 0);
                    for (i, value) in chunk.values.enumerate() {
                        assert_eq!(chunk.begin_idx + i, value + 0);
                    }
                }
            });
        }
    });
}

#[test]
fn exact_range() {
    let range = 0..8564;
    validate_exact_iter(|| range.clone().con_iter(), range.len());
    validate_exact_iter(|| range.clone().into_exact_con_iter(), range.len());
    validate_exact_iter_concurrently(|| range.clone().into_exact_con_iter());
}

#[test]
fn exact_vec() {
    let vec: Vec<_> = (0..8564).collect();
    validate_exact_iter(|| vec.clone().into_con_iter(), vec.len());
    validate_exact_iter(|| vec.clone().into_exact_con_iter(), vec.len());
    validate_exact_iter_concurrently(|| vec.clone().into_exact_con_iter());
}

#[test]
fn exact_slice() {
    let vec: Vec<_> = (0..8564).collect();
    validate_exact_iter(|| vec.con_iter(), vec.len());
    validate_exact_iter(|| vec.as_slice().into_con_iter(), vec.len());
    validate_exact_iter(|| vec.as_slice().into_exact_con_iter(), vec.len());
    validate_exact_iter_concurrently(|| vec.as_slice().into_exact_con_iter());
}

#[test]
fn exact_array() {
    let mut array = [0usize; 1587];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    validate_exact_iter(|| array.con_iter(), array.len());
    validate_exact_iter(|| array.into_con_iter(), array.len());
    validate_exact_iter(|| array.into_exact_con_iter(), array.len());
    validate_exact_iter_concurrently(|| array.into_exact_con_iter());
}
