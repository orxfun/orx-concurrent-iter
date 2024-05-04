mod atomic;

use atomic::{
    atomic_exact_fetch_n, atomic_exact_fetch_one, atomic_fetch_n, atomic_fetch_one,
    test_ids_and_values, test_values, ATOMIC_FETCH_N, ATOMIC_TEST_LEN,
};
use orx_concurrent_iter::iter::atomic_iter::*;
use orx_concurrent_iter::*;
use test_case::test_matrix;

#[test]
fn new() {
    let range = 3..10;
    let con_iter = ConIterOfRange::new(range);

    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, 3 + i);
        i += 1;
    }
    assert_eq!(i, 7);
    assert_eq!(con_iter.counter().current(), 8);
}

#[test]
fn from() {
    let range = 3..10;
    let con_iter: ConIterOfRange<_> = range.into();
    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, 3 + i);
        i += 1;
    }
    assert_eq!(i, 7);
    assert_eq!(con_iter.counter().current(), 8);
}

#[test]
fn debug() {
    let range = 3..10;
    let con_iter = ConIterOfRange::new(range);

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfRange { range: 3..10, counter: AtomicCounter { current: 0 } }"
    );

    assert_eq!(con_iter.next(), Some(3));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfRange { range: 3..10, counter: AtomicCounter { current: 1 } }"
    );
}

#[test]
fn clone() {
    let range = 3..6;
    let con_iter = ConIterOfRange::new(range);

    assert_eq!(con_iter.next(), Some(3));
    assert_eq!(1, con_iter.counter().current());

    let clone = con_iter.clone();
    assert_eq!(1, clone.counter().current());

    assert_eq!(clone.next(), Some(4));
    assert_eq!(clone.next(), Some(5));
    assert_eq!(3, clone.counter().current());

    assert_eq!(clone.next(), None);
    assert_eq!(4, clone.counter().current());

    assert_eq!(clone.next(), None);
    assert_eq!(5, clone.counter().current());

    assert_eq!(1, con_iter.counter().current());
}

#[test]
fn atomic() {
    atomic_fetch_one(ConIterOfRange::new(0..ATOMIC_TEST_LEN));
    for n in ATOMIC_FETCH_N {
        atomic_fetch_n(ConIterOfRange::new(0..ATOMIC_TEST_LEN), n);
    }
}

#[test]
fn atomic_exact() {
    atomic_exact_fetch_one(ConIterOfRange::new(0..ATOMIC_TEST_LEN));
    for n in ATOMIC_FETCH_N {
        atomic_exact_fetch_n(ConIterOfRange::new(0..ATOMIC_TEST_LEN), n);
    }
}

#[test_matrix(
    [1, 2, 8],
    [1, 2, 64, 1025, 5483]
)]
fn ids_and_values(num_threads: usize, len: usize) {
    test_values(num_threads, len, (0..len).con_iter());
    test_ids_and_values(num_threads, len, (0..len).con_iter());
}
