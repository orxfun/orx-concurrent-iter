mod atomic;

use atomic::{
    atomic_fetch_n, atomic_fetch_one, atomic_initial_len, test_ids_and_values, test_values,
    ATOMIC_FETCH_N, ATOMIC_TEST_LEN,
};
use orx_concurrent_iter::iter::atomic_iter::*;
use orx_concurrent_iter::*;
use test_case::test_matrix;

#[test]
fn new() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter = ConIterOfSlice::new(slice);
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn from() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter = ConIterOfSlice::new(slice);
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn debug() {
    let values = ['a', 'b', 'c'];
    let con_iter: ConIterOfSlice<_> = values.as_slice().into();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfSlice { slice: ['a', 'b', 'c'], counter: AtomicCounter { current: 0 } }"
    );

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfSlice { slice: ['a', 'b', 'c'], counter: AtomicCounter { current: 1 } }"
    );
}

#[test]
fn as_slice() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();
    let vec = slice.to_vec();

    let con_iter: ConIterOfSlice<_> = slice.into();

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(slice, &vec);
}

#[test]
fn clone() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter: ConIterOfSlice<_> = slice.into();

    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(1, con_iter.counter().current());

    let clone = con_iter.clone();
    assert_eq!(1, clone.counter().current());

    assert_eq!(clone.next(), Some(&'b'));
    assert_eq!(clone.next(), Some(&'c'));
    assert_eq!(3, clone.counter().current());

    assert_eq!(clone.next(), None);
    assert_eq!(4, clone.counter().current());

    assert_eq!(clone.next(), None);
    assert_eq!(5, clone.counter().current());

    assert_eq!(1, con_iter.counter().current());
}

#[test]
fn atomic() {
    let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
    atomic_fetch_one(ConIterOfSlice::new(values.as_slice()));
    for n in ATOMIC_FETCH_N {
        atomic_fetch_n(ConIterOfSlice::new(values.as_slice()), n);
    }
}

#[test]
fn atomic_exact() {
    let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
    atomic_initial_len(ConIterOfSlice::new(values.as_slice()));
}

#[test_matrix(
    [1, 2, 8],
    [1, 8, 64, 1025, 5483]
)]
fn ids_and_values(num_threads: usize, len: usize) {
    let values: Vec<_> = (0..len).collect();
    let slice = values.as_slice();
    test_values(num_threads, len, slice.into_con_iter());
    test_ids_and_values(num_threads, len, slice.into_con_iter());
}
