use super::indexers::{GeneralJaggedIndexer, MatrixIndexer};
use crate::implementations::jagged_arrays::reference::{
    raw_jagged_ref::RawJaggedRef, slice_iter::RawJaggedSliceIterRef,
};
use std::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use test_case::test_matrix;

// matrix

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

#[test_matrix(
    [true, false],
    [true, false]
)]
fn raw_jagged_iter_ref_matrix(consume_taken: bool, consume_remaining: bool) {
    let n = 4;
    let len = n * n;
    let data = get_matrix(n);

    let jagged = || RawJaggedRef::new(data.as_slice(), MatrixIndexer::new(n), Some(n * n));

    for num_taken in 0..len {
        let jagged = jagged();

        let iter = RawJaggedSliceIterRef::new(jagged.jagged_slice(0, num_taken));
        if consume_taken {
            let from_jagged: Vec<_> = iter.cloned().collect();
            let expected: Vec<_> = (0..num_taken).map(|x| x.to_string()).collect();
            assert_eq!(from_jagged, expected);
        }

        let remaining = RawJaggedSliceIterRef::new(jagged.jagged_slice(num_taken, len));
        if consume_remaining {
            let remaining: Vec<_> = remaining.cloned().collect();
            let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
            assert_eq!(remaining, expected_remaining);
        }
    }
}

// jagged

fn get_jagged() -> Vec<Vec<String>> {
    let jagged = vec![vec![0, 1], vec![2, 3, 4], vec![5], vec![6, 7, 8, 9]];
    jagged
        .into_iter()
        .map(|x| x.into_iter().map(|x| x.to_string()).collect::<Vec<_>>())
        .collect()
}

#[test_matrix(
    [true, false],
    [true, false]
)]
fn raw_jagged_iter_ref_jagged(consume_taken: bool, consume_remaining: bool) {
    let len = 10;
    let data = get_jagged();

    let jagged = || RawJaggedRef::new(data.as_slice(), GeneralJaggedIndexer, Some(10));

    for num_taken in 0..len {
        let jagged = jagged();

        let iter = RawJaggedSliceIterRef::new(jagged.jagged_slice(0, num_taken));
        if consume_taken {
            let from_jagged: Vec<_> = iter.cloned().collect();
            let expected: Vec<_> = (0..num_taken).map(|x| x.to_string()).collect();
            assert_eq!(from_jagged, expected);
        }

        let remaining = RawJaggedSliceIterRef::new(jagged.jagged_slice(num_taken, len));
        if consume_remaining {
            let remaining: Vec<_> = remaining.cloned().collect();
            let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
            assert_eq!(remaining, expected_remaining);
        }
    }
}
