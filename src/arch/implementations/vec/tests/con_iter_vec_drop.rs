use crate::{
    concurrent_iter::ConcurrentIter,
    enumeration::{Enumerated, Enumeration, Regular},
    implementations::vec::con_iter_vec::ConIterVec,
};
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 4735;

#[test_matrix([Regular, Enumerated])]
fn empty<E: Enumeration>(_: E) {
    let _ = ConIterVec::<String, E>::default();
}

#[test_matrix([Regular, Enumerated])]
fn all_consumed<E: Enumeration>(_: E) {
    let cap = N + 19;
    let mut vec = Vec::with_capacity(cap);
    for i in 0..N {
        vec.push(i.to_string());
    }

    let iter = ConIterVec::<_, E>::new(vec);
    while let Some(_) = iter.next() {}
}

#[test_matrix([Regular, Enumerated])]
fn partially_consumed<E: Enumeration>(_: E) {
    let cap = N + 19;
    let mut vec = Vec::with_capacity(cap);
    for i in 0..N {
        vec.push(i.to_string());
    }

    let iter = ConIterVec::<_, E>::new(vec);
    for _ in 0..(N / 2) {
        _ = iter.next().unwrap();
    }
}
