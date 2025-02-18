use crate::{concurrent_iter::ConcurrentIter, ConcurrentIterable, IntoConcurrentIter};
use orx_concurrent_bag::ConcurrentBag;

#[test]
fn slice_into_con_iter() {
    let (nt, n) = (2, 177);
    let vec: Vec<_> = (0..n).map(|x| (x + 10).to_string()).collect();
    let slice = vec.as_slice();
    let iter = slice.into_concurrent_iter();

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

    let mut expected: Vec<_> = (0..n).map(|i| &slice[i]).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}

#[test]
fn slice_con_iter() {
    let (nt, n) = (2, 177);
    let vec: Vec<_> = (0..n).map(|x| (x + 10).to_string()).collect();
    let slice = vec.as_slice();
    let iter = slice.concurrent_iter();

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

    let mut expected: Vec<_> = (0..n).map(|i| &slice[i]).collect();
    expected.sort();
    let mut collected = bag.into_inner().to_vec();
    collected.sort();

    assert_eq!(expected, collected);
}
