use crate::{concurrent_iter::ConcurrentIter, IterIntoConcurrentIter};
use orx_concurrent_bag::ConcurrentBag;

#[test]
fn ref_iter_as_iter_into_concurrent_iter() {
    let (nt, n) = (2, 177);
    let vec: Vec<_> = (0..n).map(|x| (x + 10).to_string()).collect();
    let iter = vec.iter().filter(|x| x.as_str() != "abc");
    let con_iter = iter.iter_into_concurrent_iter();

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = con_iter.next() {
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
fn owned_iter_as_iter_into_concurrent_iter() {
    let (nt, n) = (2, 177);
    let vec: Vec<_> = (0..n).map(|x| (x + 10).to_string()).collect();
    let iter = vec.into_iter().filter(|x| x.as_str() != "abc");
    let con_iter = iter.iter_into_concurrent_iter();

    let bag = ConcurrentBag::new();
    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = con_iter.next() {
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
