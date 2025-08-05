use crate::{IntoConcurrentIter, concurrent_iter::ConcurrentIter};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;

#[test]
fn slice_as_into_concurrent_iter() {
    let (nt, n) = (2, 177);
    let mut vec: Vec<_> = (0..n).collect();
    let slice = vec.as_mut_slice();

    let iter = slice.into_con_iter();

    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    *x += 1;
                }
            });
        }
    });

    let expected: Vec<_> = (1..(n + 1)).collect();
    assert_eq!(expected, vec);
}
