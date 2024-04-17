mod atomic;

use atomic::{
    atomic_exact_fetch_n, atomic_exact_fetch_one, atomic_fetch_n, atomic_fetch_one, ATOMIC_FETCH_N,
    ATOMIC_TEST_LEN,
};
use orx_concurrent_iter::*;

#[test]
fn new() {
    let values = ['a', 'b', 'c'];
    let con_iter = ConIterOfArray::new(values);

    let mut collected = vec![];
    while let Some(x) = con_iter.next() {
        collected.push(x);
    }

    assert_eq!(collected, vec!['a', 'b', 'c']);
}

#[test]
fn from() {
    let values = ['a', 'b', 'c'];
    let con_iter: ConIterOfArray<3, _> = values.into();

    let mut collected = vec![];
    while let Some(x) = con_iter.next() {
        collected.push(x);
    }

    assert_eq!(collected, vec!['a', 'b', 'c']);
}

#[test]
fn debug() {
    let values = ['a', 'b', 'c'];
    let con_iter: ConIterOfArray<3, _> = values.into();

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfArray { array: UnsafeCell { .. }, counter: AtomicCounter { current: 0 } }"
    );

    assert_eq!(con_iter.next(), Some('a'));

    assert_eq!(
        format!("{:?}", con_iter),
        "ConIterOfArray { array: UnsafeCell { .. }, counter: AtomicCounter { current: 1 } }"
    );
}

#[test]
fn atomic() {
    let mut values = [0usize; ATOMIC_TEST_LEN];
    for (i, elem) in values.iter_mut().enumerate() {
        *elem = i;
    }
    atomic_fetch_one(ConIterOfArray::new(values));
    for n in ATOMIC_FETCH_N {
        atomic_fetch_n(ConIterOfArray::new(values), n);
    }
}

#[test]
fn atomic_exact() {
    let mut values = [0usize; ATOMIC_TEST_LEN];
    for (i, elem) in values.iter_mut().enumerate() {
        *elem = i;
    }
    atomic_exact_fetch_one(ConIterOfArray::new(values));
    for n in ATOMIC_FETCH_N {
        atomic_exact_fetch_n(ConIterOfArray::new(values), n);
    }
}
