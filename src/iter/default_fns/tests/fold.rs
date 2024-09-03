use crate::iter::default_fns;
use crate::*;
use test_case::test_matrix;

#[test_matrix(
    [8, 64, 256],
    [4, 8, 16]
)]
fn any_fold(len: usize, chunk_size: usize) {
    let numbers: Vec<_> = (0..len).collect();
    let expected: i64 = numbers.iter().skip(1).take(len - 1).sum::<usize>() as i64;

    let iter = numbers.iter().skip(1).take(len - 1);
    let con_iter = iter.into_con_iter();
    let sum = default_fns::fold::fold_x(&con_iter, chunk_size, |a, b| a + *b as i64, 0i64);

    assert_eq!(sum, expected);
}

#[test_matrix(
    [8, 64, 256],
    [4, 8, 16]
)]
fn exact_fold(len: usize, chunk_size: usize) {
    let numbers: Vec<_> = (0..len).collect();
    let expected = numbers.iter().sum::<usize>() as i64;

    let sum =
        default_fns::fold::fold_x(&numbers.con_iter(), chunk_size, |a, b| a + *b as i64, 0i64);

    assert_eq!(sum, expected);
}

// panics
#[test]
#[should_panic]
fn panics_fold_with_zero_chunk() {
    let numbers: Vec<_> = (0..64).collect();
    let iter = numbers.iter().skip(1).take(8);
    let con_iter = iter.into_con_iter();
    let _ = default_fns::fold::fold_x(&con_iter, 0, |a, b| a + *b as i64, 0i64);
}
