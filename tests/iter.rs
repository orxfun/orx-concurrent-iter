mod atomic;

use atomic::{atomic_fetch_n, atomic_fetch_one, ATOMIC_FETCH_N, ATOMIC_TEST_LEN};
use orx_concurrent_iter::*;

#[test]
fn new() {
    let values = ['a', 'b', 'c'];

    let con_iter = ConIterOfIter::new(values.iter());

    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, &values[i]);
        i += 1;
    }
    assert_eq!(i, values.len());
}

#[test]
fn from() {
    let values = ['a', 'b', 'c'];

    let con_iter: ConIterOfIter<_, _> = values.iter().into();

    let mut i = 0;
    while let Some(x) = con_iter.next() {
        assert_eq!(x, &values[i]);
        i += 1;
    }
    assert_eq!(i, values.len());
}

#[test]
fn debug() {
    let values = ['a', 'b', 'c'];
    let con_iter: ConIterOfIter<_, _> = values.iter().into();

    assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfIter { iter: UnsafeCell { .. }, initial_len: Some(3), reserved_counter: AtomicCounter { current: 0 }, yielded_counter: AtomicCounter { current: 0 }, completed: false }"
        );

    assert_eq!(con_iter.next(), Some(&'a'));

    assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfIter { iter: UnsafeCell { .. }, initial_len: Some(3), reserved_counter: AtomicCounter { current: 1 }, yielded_counter: AtomicCounter { current: 1 }, completed: false }"
        );

    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));

    assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfIter { iter: UnsafeCell { .. }, initial_len: Some(3), reserved_counter: AtomicCounter { current: 3 }, yielded_counter: AtomicCounter { current: 3 }, completed: false }"
        );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfIter { iter: UnsafeCell { .. }, initial_len: Some(3), reserved_counter: AtomicCounter { current: 4 }, yielded_counter: AtomicCounter { current: 3 }, completed: true }"
        );

    assert_eq!(con_iter.next(), None);

    assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfIter { iter: UnsafeCell { .. }, initial_len: Some(3), reserved_counter: AtomicCounter { current: 5 }, yielded_counter: AtomicCounter { current: 3 }, completed: true }"
        );
}

#[test]
fn atomic() {
    let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
    atomic_fetch_one(ConIterOfIter::new(values.iter()));
    for n in ATOMIC_FETCH_N {
        atomic_fetch_n(ConIterOfIter::new(values.iter()), n);
    }
}
