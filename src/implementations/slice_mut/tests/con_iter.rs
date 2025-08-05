use crate::{
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::slice_mut::ConIterSliceMut, pullers::ChunkPuller,
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
    let mut vec: Vec<_> = (0..6).collect();
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    assert_eq!(iter.next(), Some(&mut 0));
    assert_eq!(iter.next_with_idx(), Some((1, &mut 1)));
    assert_eq!(iter.next(), Some(&mut 2));
    assert_eq!(iter.next_with_idx(), Some((3, &mut 3)));
    assert_eq!(iter.next(), Some(&mut 4));
    assert_eq!(iter.next_with_idx(), Some((5, &mut 5)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_with_idx(), None);
}

#[test]
fn size_hint() {
    let mut n = 25;
    let mut vec = new_vec(n, |x| (x + 10).to_string());
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    for _ in 0..10 {
        assert_eq!(iter.size_hint(), (n, Some(n)));
        let _ = iter.next();
        n -= 1;
    }

    let mut chunks_iter = iter.chunk_puller(7);

    assert_eq!(iter.size_hint(), (n, Some(n)));
    assert_eq!(iter.len(), n);
    let _ = chunks_iter.pull();
    n -= 7;

    assert_eq!(iter.size_hint(), (n, Some(n)));
    assert_eq!(iter.len(), n);
    let _ = chunks_iter.pull();
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let _ = chunks_iter.pull();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let _ = chunks_iter.pull();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));

    let _ = iter.next();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn size_hint_skip_to_end() {
    let n = 25;
    let mut vec = new_vec(n, |x| (x + 10).to_string());
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    for _ in 0..10 {
        let _ = iter.next();
    }
    let mut chunks_iter = iter.chunk_puller(7);
    let _ = chunks_iter.pull();

    assert_eq!(iter.len(), 8);

    iter.skip_to_end();
    assert_eq!(iter.len(), 0);
}

#[test_matrix([1, 2, 4])]
fn empty(nt: usize) {
    let mut vec = Vec::<String>::new();
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                assert!(iter.next().is_none());
                assert!(iter.next().is_none());

                let mut puller = iter.chunk_puller(5);
                assert!(puller.pull().is_none());
                assert!(puller.pull().is_none());

                let mut iter = iter.chunk_puller(5).flattened();
                assert!(iter.next().is_none());
                assert!(iter.next().is_none());
            });
        }
    });
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn next(n: usize, nt: usize) {
    let mut vec = new_vec(n, |x| (x + 10).to_string());
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next() {
                    _ = iter.size_hint();
                    x.push('!');
                }
            });
        }
    });

    let expected = new_vec(n, |x| format!("{}!", x + 10));
    assert_eq!(expected, vec);
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn next_with_idx(n: usize, nt: usize) {
    let mut vec = new_vec(n, |x| (x + 10).to_string());
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                while let Some(x) = iter.next_with_idx() {
                    _ = iter.size_hint();
                    assert_eq!(x.0 + 10, x.1.parse::<usize>().unwrap());
                    x.1.push('!');
                }
            });
        }
    });

    let expected = new_vec(n, |x| format!("{}!", x + 10));
    assert_eq!(expected, vec);
}

#[test_matrix([0, 1, N], [1, 2, 4])]
fn item_puller(n: usize, nt: usize) {
    let mut vec = new_vec(n, |x| (x + 10).to_string());
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                for x in iter.item_puller() {
                    _ = iter.size_hint();
                    x.push('!');
                }
            });
        }
    });

    let expected = new_vec(n, |x| format!("{}!", x + 10));
    assert_eq!(expected, vec);
}

#[test_matrix( [0, 1, N], [1, 2, 4])]
fn item_puller_with_idx(n: usize, nt: usize) {
    let mut vec = new_vec(n, |x| (x + 10).to_string());
    let slice = vec.as_mut_slice();
    let iter = ConIterSliceMut::new(slice);

    let num_spawned = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                num_spawned.push(true);
                while num_spawned.len() < nt {} // allow all threads to be spawned

                for x in iter.item_puller_with_idx() {
                    _ = iter.size_hint();
                    assert_eq!(x.0 + 10, x.1.parse::<usize>().unwrap());
                    x.1.push('!');
                }
            });
        }
    });

    let expected = new_vec(n, |x| format!("{}!", x + 10));
    assert_eq!(expected, vec);
}

// #[test_matrix([0, 1, N], [1, 2, 4])]
// fn chunk_puller(n: usize, nt: usize) {
//     let mut vec = new_vec(n, |x| (x + 10).to_string());
//     let slice = vec.as_mut_slice();
//     let iter = ConIterSliceMut::new(slice);

//     let bag = ConcurrentBag::new();
//     let num_spawned = ConcurrentBag::new();
//     std::thread::scope(|s| {
//         for _ in 0..nt {
//             s.spawn(|| {
//                 num_spawned.push(true);
//                 while num_spawned.len() < nt {} // allow all threads to be spawned

//                 let mut puller = iter.chunk_puller(7);

//                 while let Some(chunk) = puller.pull() {
//                     assert!(chunk.len() <= 7);
//                     for x in chunk {
//                         bag.push(x);
//                     }
//                 }
//             });
//         }
//     });

//     let mut expected: Vec<_> = (0..n).map(|i| &vec[i]).collect();
//     expected.sort();
//     let mut collected = bag.into_inner().to_vec();
//     collected.sort();

//     assert_eq!(expected, collected);
// }

// #[test_matrix([0, 1, N], [1, 2, 4])]
// fn chunk_puller_with_idx(n: usize, nt: usize) {
//     let mut vec = new_vec(n, |x| (x + 10).to_string());
//     let slice = vec.as_mut_slice();
//     let iter = ConIterSliceMut::new(slice);

//     let bag = ConcurrentBag::new();
//     let num_spawned = ConcurrentBag::new();
//     std::thread::scope(|s| {
//         for _ in 0..nt {
//             s.spawn(|| {
//                 num_spawned.push(true);
//                 while num_spawned.len() < nt {} // allow all threads to be spawned

//                 let mut puller = iter.chunk_puller(7);

//                 while let Some((begin_idx, chunk)) = puller.pull_with_idx() {
//                     assert!(chunk.len() <= 7);
//                     for (i, x) in chunk.enumerate() {
//                         bag.push((begin_idx + i, x));
//                     }
//                 }
//             });
//         }
//     });

//     let mut expected: Vec<_> = (0..n).map(|i| (i, &vec[i])).collect();
//     expected.sort();
//     let mut collected = bag.into_inner().to_vec();
//     collected.sort();

//     assert_eq!(expected, collected);
// }

// #[test_matrix([0, 1, N], [1, 2, 4])]
// fn flattened_chunk_puller(n: usize, nt: usize) {
//     let mut vec = new_vec(n, |x| (x + 10).to_string());
//     let slice = vec.as_mut_slice();
//     let iter = ConIterSliceMut::new(slice);

//     let bag = ConcurrentBag::new();
//     let num_spawned = ConcurrentBag::new();
//     std::thread::scope(|s| {
//         for _ in 0..nt {
//             s.spawn(|| {
//                 num_spawned.push(true);
//                 while num_spawned.len() < nt {} // allow all threads to be spawned

//                 for x in iter.chunk_puller(7).flattened() {
//                     bag.push(x);
//                 }
//             });
//         }
//     });

//     let mut expected: Vec<_> = (0..n).map(|i| &vec[i]).collect();
//     expected.sort();
//     let mut collected = bag.into_inner().to_vec();
//     collected.sort();

//     assert_eq!(expected, collected);
// }

// #[test_matrix([0, 1, N], [1, 2, 4])]
// fn flattened_chunk_puller_with_idx(n: usize, nt: usize) {
//     let mut vec = new_vec(n, |x| (x + 10).to_string());
//     let slice = vec.as_mut_slice();
//     let iter = ConIterSliceMut::new(slice);

//     let bag = ConcurrentBag::new();
//     let num_spawned = ConcurrentBag::new();
//     std::thread::scope(|s| {
//         for _ in 0..nt {
//             s.spawn(|| {
//                 num_spawned.push(true);
//                 while num_spawned.len() < nt {} // allow all threads to be spawned

//                 for x in iter.chunk_puller(7).flattened_with_idx() {
//                     bag.push(x);
//                 }
//             });
//         }
//     });

//     let mut expected: Vec<_> = (0..n).map(|i| (i, &vec[i])).collect();
//     expected.sort();
//     let mut collected = bag.into_inner().to_vec();
//     collected.sort();

//     assert_eq!(expected, collected);
// }

// #[test_matrix([0, 1, N], [1, 2, 4])]
// fn skip_to_end(n: usize, nt: usize) {
//     let mut vec = new_vec(n, |x| (x + 10).to_string());
//     let slice = vec.as_mut_slice();
//     let iter = ConIterSliceMut::new(slice);

//     let until = n / 2;

//     let bag = ConcurrentBag::new();
//     let num_spawned = ConcurrentBag::new();
//     let con_num_spawned = &num_spawned;
//     let con_bag = &bag;
//     let con_iter = &iter;
//     std::thread::scope(|s| {
//         for t in 0..nt {
//             s.spawn(move || {
//                 con_num_spawned.push(true);
//                 while con_num_spawned.len() < nt {} // allow all threads to be spawned

//                 match t % 2 {
//                     0 => {
//                         while let Some(num) = con_iter.next() {
//                             match num.parse::<usize>().expect("") < until + 10 {
//                                 true => _ = con_bag.push(num),
//                                 false => con_iter.skip_to_end(),
//                             }
//                         }
//                     }
//                     _ => {
//                         for num in con_iter.chunk_puller(7).flattened() {
//                             match num.parse::<usize>().expect("") < until + 10 {
//                                 true => _ = con_bag.push(num),
//                                 false => con_iter.skip_to_end(),
//                             }
//                         }
//                     }
//                 }
//             });
//         }
//     });

//     let mut expected: Vec<_> = (0..until).map(|i| &vec[i]).collect();
//     expected.sort();
//     let mut collected = bag.into_inner().to_vec();
//     collected.sort();

//     assert_eq!(expected, collected);
// }

// #[test_matrix([0, 1, N], [1, 2, 4], [0, N / 2, N])]
// fn into_seq_iter(n: usize, nt: usize, until: usize) {
//     let mut vec = new_vec(n, |x| (x + 10).to_string());
//     let slice = vec.as_mut_slice();
//     let iter = ConIterSliceMut::new(slice);

//     let bag = ConcurrentBag::new();
//     let num_spawned = ConcurrentBag::new();
//     let con_num_spawned = &num_spawned;
//     let con_bag = &bag;
//     let con_iter = &iter;
//     if until > 0 {
//         std::thread::scope(|s| {
//             for t in 0..nt {
//                 s.spawn(move || {
//                     con_num_spawned.push(true);
//                     while con_num_spawned.len() < nt {} // allow all threads to be spawned

//                     match t % 2 {
//                         0 => {
//                             while let Some(num) = con_iter.next() {
//                                 con_bag.push(num.clone());
//                                 if num.parse::<usize>().expect("") >= until + 10 {
//                                     break;
//                                 }
//                             }
//                         }
//                         _ => {
//                             let mut iter = con_iter.chunk_puller(7);
//                             while let Some(chunk) = iter.pull() {
//                                 let mut do_break = false;
//                                 for num in chunk {
//                                     con_bag.push(num.clone());
//                                     if num.parse::<usize>().expect("") >= until + 10 {
//                                         do_break = true;
//                                     }
//                                 }
//                                 if do_break {
//                                     break;
//                                 }
//                             }
//                         }
//                     }
//                 });
//             }
//         });
//     }

//     let iter = iter.into_seq_iter();
//     let remaining: Vec<_> = iter.cloned().collect();
//     let collected = bag.into_inner().to_vec();
//     let mut all: Vec<_> = collected.into_iter().chain(remaining).collect();
//     all.sort();

//     let mut expected: Vec<_> = (0..n).map(|i| vec[i].clone()).collect();
//     expected.sort();

//     assert_eq!(all, expected);
// }
