use crate::concurrent_iter::ConcurrentIter;
use alloc::{string::ToString, vec::Vec};
use orx_concurrent_bag::ConcurrentBag;

#[test]
fn vec_as_into_concurrent_iter() {
    use crate::IntoConcurrentIter;
    let (nt, n) = (2, 37);
    let vec: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();

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
fn vec_as_concurrent_iterable() {
    use crate::ConcurrentIterable;
    let (nt, n) = (2, 37);
    let vec: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();

    let iter = (&vec).con_iter();

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

#[test]
fn vec_as_concurrent_collection() {
    use crate::ConcurrentCollection;
    let (nt, n) = (2, 37);
    let vec: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();

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

#[test]
fn vec_as_concurrent_collection_mut() {
    use crate::ConcurrentCollectionMut;
    let (nt, n) = (2, 37);
    let mut vec: Vec<_> = (0..n).map(|i| (i + 10).to_string()).collect();

    let iter = vec.con_iter_mut();

    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    x.push('!');
                }
            });
        }
    });

    let expected: Vec<_> = (0..n).map(|i| format!("{}!", i + 10)).collect();
    assert_eq!(expected, vec);
}
