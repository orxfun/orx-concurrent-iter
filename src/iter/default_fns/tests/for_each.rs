use crate::iter::default_fns;
use crate::*;
use test_case::test_matrix;

#[test_matrix(
    [8, 64, 256],
    [4, 8, 16]
)]
fn any_for_each(len: usize, chunk_size: usize) {
    let numbers: Vec<_> = (0..len).collect();
    let expected: usize = numbers.iter().skip(1).take(len - 1).sum();
    let mut sum = 0;

    let iter = numbers.iter().skip(1).take(len - 1);
    let con_iter = iter.into_con_iter();
    default_fns::for_each::for_each(&con_iter, chunk_size, |x| {
        sum += x;
    });

    assert_eq!(sum, expected);
}

#[test_matrix(
    [1, 8, 64, 256],
    [4, 8, 16]
)]
fn any_for_each_with_ids(len: usize, chunk_size: usize) {
    let numbers: Vec<_> = (0..len).collect();
    let expected: usize = numbers.iter().sum();
    let mut sum = 0;
    let mut indices = 0;

    let con_iter = numbers.into_con_iter();
    default_fns::for_each::for_each_with_ids(&con_iter, chunk_size, |i, x| {
        sum += x;
        indices -= i as i64;
    });

    assert_eq!(sum, expected);
    assert_eq!(indices, -(expected as i64));
}

#[test_matrix(
    [8, 64, 256],
    [4, 8, 16]
)]
fn exact_for_each(len: usize, chunk_size: usize) {
    let numbers: Vec<_> = (0..len).collect();
    let expected: usize = numbers.iter().sum();
    let mut sum = 0;

    default_fns::for_each::for_each(&numbers.con_iter(), chunk_size, |x| {
        sum += x;
    });

    assert_eq!(sum, expected);
}

#[test_matrix(
    [8, 64, 256],
    [4, 8, 16]
)]
fn exact_for_each_with_ids(len: usize, chunk_size: usize) {
    let numbers: Vec<_> = (0..len).collect();
    let expected: usize = numbers.iter().sum();
    let mut sum = 0;
    let mut indices = 0;

    default_fns::for_each::for_each_with_ids(&numbers.con_iter(), chunk_size, |i, x| {
        sum += x;
        indices -= i as i64;
    });

    assert_eq!(sum, expected);
    assert_eq!(indices, -(expected as i64));
}

// panics
#[test]
#[should_panic]
fn panics_any_for_each() {
    let numbers: Vec<_> = (0..64).collect();
    let mut sum = 0;
    let iter = numbers.iter().skip(1).take(8);
    let con_iter = iter.into_con_iter();
    default_fns::for_each::for_each(&con_iter, 0, |x| {
        sum += x;
    });
}

#[test]
#[should_panic]
fn panics_any_for_each_with_ids() {
    let numbers: Vec<_> = (0..64).collect();
    let mut sum = 0;
    let iter = numbers.iter().skip(1).take(8);
    let con_iter = iter.into_con_iter();
    default_fns::for_each::for_each_with_ids(&con_iter, 0, |_i, x| {
        sum += x;
    });
}

#[test]
#[should_panic]
fn panics_exact_for_each() {
    let numbers: Vec<_> = (0..64).collect();
    let mut sum = 0;
    default_fns::for_each::for_each(&numbers.con_iter(), 0, |x| {
        sum += x;
    });
}

#[test]
#[should_panic]
fn panics_exact_for_each_with_ids() {
    let numbers: Vec<_> = (0..64).collect();
    let mut sum = 0;
    default_fns::for_each::for_each_with_ids(&numbers.con_iter(), 0, |_i, x| {
        sum += x;
    });
}
