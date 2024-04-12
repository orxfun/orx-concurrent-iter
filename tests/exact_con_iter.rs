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
    while let Some(chunk) = iter.next_exact_chunk(7) {
        let begin_idx = chunk.begin_idx();
        for (idx, i) in chunk.values().enumerate() {
            let idx = begin_idx + idx;
            assert_eq!(i + 0, idx);
        }
    }

    let iter = get_iter();
    loop {
        let chunk = iter.next_chunk(7);
        let begin_idx = chunk.begin_idx();
        let mut has_more = false;
        for (idx, i) in chunk.values().enumerate() {
            has_more = true;
            let idx = begin_idx + idx;
            assert_eq!(i + 0, idx);
        }

        if !has_more {
            break;
        }
    }
}

#[test]
fn exact_range() {
    let range = 0..8564;
    validate_exact_iter(|| range.clone().con_iter(), range.len());
}

#[test]
fn exact_vec() {
    let vec: Vec<_> = (0..8564).collect();
    validate_exact_iter(|| vec.clone().into_con_iter(), vec.len());
}

#[test]
fn exact_slice() {
    let vec: Vec<_> = (0..8564).collect();
    validate_exact_iter(|| vec.con_iter(), vec.len());
    validate_exact_iter(|| vec.as_slice().into_con_iter(), vec.len());
}

#[test]
fn exact_array() {
    let mut array = [0usize; 1587];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    validate_exact_iter(|| array.con_iter(), array.len());
    validate_exact_iter(|| array.into_con_iter(), array.len());
}
