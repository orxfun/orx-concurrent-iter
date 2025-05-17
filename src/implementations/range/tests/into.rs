use crate::{
    IntoConcurrentIter, concurrent_iter::ConcurrentIter, concurrent_iterable::ConcurrentIterable,
};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;

#[test]
fn range_as_into_concurrent_iter() {
    let (nt, n) = (2, 177);
    let range = 10..(10 + n);
    let iter = range.into_con_iter();

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    bag.push(x);
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..n).map(|i| 10 + i).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test]
fn range_as_concurrent_iterable() {
    let (nt, n) = (2, 177);
    let range = 10..(10 + n);
    let iter = range.con_iter();

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    bag.push(x);
                }
            });
        }
    });

    let mut expected: Vec<_> = (0..n).map(|i| i + 10).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}
