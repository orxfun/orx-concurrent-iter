use crate::{
    IntoConcurrentIter, chain::con_iter::Chain, concurrent_iter::ConcurrentIter,
    exact_size_concurrent_iter::ExactSizeConcurrentIter, pullers::ChunkPuller,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 4735;

fn new_vec(n: usize, elem: impl Fn(usize) -> String) -> Vec<String> {
    let mut vec = Vec::with_capacity(n + 17);
    for i in 0..n {
        vec.push(elem(i));
    }
    vec
}

#[test]
fn enumeration() {
    let v1: Vec<_> = (0..3).collect();
    let i1 = v1.into_con_iter();

    let v2: Vec<_> = (3..6).collect();
    let s2 = v2.as_slice();
    let i2 = s2.into_con_iter().copied();

    let iter = Chain::new(i1, i2);

    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_with_idx(), Some((1, 1)));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_with_idx(), Some((3, 3)));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next_with_idx(), Some((5, 5)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
}

// #[test]
// fn size_hint() {
//     let n1 = 12;
//     let n2 = 13;
//     let mut n = n1 + n2;
//     let v1 = new_vec(n1, |x| (x + 10).to_string());
//     let v2 = new_vec(n2, |x| (x + 10).to_string());
//     let iter = Chain::new(v1.into_con_iter(), v2.into_con_iter());

//     for _ in 0..10 {
//         assert_eq!(iter.size_hint(), (n, Some(n)));
//         let _ = iter.next();
//         n -= 1;
//     }

//     let mut chunks_iter = iter.chunk_puller(7);

//     assert_eq!(iter.size_hint(), (n, Some(n)));
//     assert_eq!(iter.len(), n);
//     let _ = chunks_iter.pull();
//     n -= 7;

//     assert_eq!(iter.size_hint(), (n, Some(n)));
//     assert_eq!(iter.len(), n);
//     let _ = chunks_iter.pull();
//     assert_eq!(iter.size_hint(), (1, Some(1)));

//     let _ = chunks_iter.pull();
//     assert_eq!(iter.len(), 0);
//     assert_eq!(iter.size_hint(), (0, Some(0)));

//     let _ = chunks_iter.pull();
//     assert_eq!(iter.len(), 0);
//     assert_eq!(iter.size_hint(), (0, Some(0)));

//     let _ = iter.next();
//     assert_eq!(iter.len(), 0);
//     assert_eq!(iter.size_hint(), (0, Some(0)));
// }
