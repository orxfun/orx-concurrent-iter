mod atomic;

use atomic::{
    atomic_exact_fetch_n, atomic_exact_fetch_one, atomic_fetch_n, atomic_fetch_one, ATOMIC_FETCH_N,
    ATOMIC_TEST_LEN,
};
use orx_concurrent_iter::*;

#[test]
fn new() {
    let values = vec!['a', 'b', 'c'];
    let con_iter = ConIterOfVec::new(values);

    let mut collected = vec![];
    while let Some(x) = con_iter.next() {
        collected.push(x);
    }

    assert_eq!(collected, vec!['a', 'b', 'c']);
}

#[test]
fn from() {
    let values = vec!['a', 'b', 'c'];
    let con_iter: ConIterOfVec<_> = values.into();

    let mut collected = vec![];
    while let Some(x) = con_iter.next() {
        collected.push(x);
    }

    assert_eq!(collected, vec!['a', 'b', 'c']);
}

#[test]
fn debug() {
    let values = vec!['a', 'b', 'c'];
    let con_iter: ConIterOfVec<_> = values.into();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfVec { vec: UnsafeCell { .. }, vec_len: 3, counter: AtomicCounter { current: 0 } }"
    );

    assert_eq!(con_iter.next(), Some('a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfVec { vec: UnsafeCell { .. }, vec_len: 3, counter: AtomicCounter { current: 1 } }"
    );
}

#[test]
fn atomic() {
    let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
    atomic_fetch_one(ConIterOfVec::new(values.clone()));
    for n in ATOMIC_FETCH_N {
        atomic_fetch_n(ConIterOfVec::new(values.clone()), n);
    }
}

#[test]
fn atomic_exact() {
    let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
    atomic_exact_fetch_one(ConIterOfVec::new(values.clone()));
    for n in ATOMIC_FETCH_N {
        atomic_exact_fetch_n(ConIterOfVec::new(values.clone()), n);
    }
}
