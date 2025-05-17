use crate::{
    IntoConcurrentIter, concurrent_collection::ConcurrentCollection,
    concurrent_iter::ConcurrentIter,
};
use alloc::{collections::VecDeque, string::ToString, vec::Vec};
use orx_concurrent_bag::ConcurrentBag;

#[test]
fn vec_deque_as_into_concurrent_iter() {
    let (nt, n) = (2, 177);
    let vec: VecDeque<_> = (0..n).map(|i| (i + 10).to_string()).collect();

    let iter = vec.into_con_iter();

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

    let mut expected: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test]
fn vec_deque_as_concurrent_iterable() {
    let (nt, n) = (2, 177);
    let vec: VecDeque<_> = (0..n).map(|i| (i + 10).to_string()).collect();

    let iter = vec.con_iter();

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

    let mut expected: Vec<_> = (0..n).map(|i| &vec[i]).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}
