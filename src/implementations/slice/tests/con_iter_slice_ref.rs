use crate::{
    chunk_puller::ChunkPuller,
    concurrent_iter::ConcurrentIter,
    enumeration::{Enumerated, Enumeration, Regular},
    implementations::slice::con_iter_slice_ref::ConIterSliceRef,
};
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

// #[test_matrix([Regular, Enumerated])]
// fn next<K: Enumeration>(_: K) {
//     let n = 123;
//     let vec: Vec<_> = (0..n).map(|x| x + 10).collect();
//     let slice = vec.as_slice();
//     let con_iter = ConIterSliceRef::<_, K>::new(slice);
//     for i in 0..n {
//         let x = i + 10;
//         let next = con_iter.next().unwrap();
//         assert!(K::eq_next(next, K::new_next(i, &x)));
//     }
// }

// #[test_matrix([Regular, Enumerated])]
// fn in_chunks2<K: Enumeration>(_: K) {
//     let n = 123;
//     let vec: Vec<_> = (0..n).map(|x| x + 10).collect();
//     let slice = vec.as_slice();
//     let con_iter = ConIterSliceRef::<_, K>::new(slice);
//     let mut puller = con_iter.in_chunks(5);
//     let mut i = 0;
//     while let Some(x) = puller.pull() {
//         let (begin_idx, iter) = K::destruct_next(x);
//         // assert!(K::eq_begin_idx(begin_idx, i));

//         match i {
//             120 => assert_eq!(iter.len(), 3),
//             _ => assert_eq!(iter.len(), 5),
//         };
//         for x in iter {
//             assert_eq!(*x, i + 10);
//             i += 1;
//         }
//     }
// }

// #[test_matrix([Regular, Enumerated])]
// fn chunks_iter<K: Enumeration>(_: K) {
//     let n = 123;
//     let vec: Vec<_> = (0..n).map(|x| x + 10).collect();
//     let slice = vec.as_slice();
//     let con_iter = ConIterSliceRef::<_, K>::new(slice);
//     let iter = con_iter.in_chunks(5).flatten();
//     let mut i = 0;
//     for (idx, x) in iter.map(K::destruct_next) {
//         assert!(K::validate_begin_idx(idx, |idx| idx == i));
//         assert_eq!(*x, i + 10);
//         i += 1;
//     }
// }

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn empty_slice<K: Enumeration>(_: K, nt: usize) {
    let vec = Vec::<String>::new();
    let slice = vec.as_slice();
    let con_iter = ConIterSliceRef::<String, K>::new(slice);

    std::thread::scope(|s| {
        for _ in 0..nt {
            s.spawn(|| {
                assert!(con_iter.next().is_none());
                assert!(con_iter.next().is_none());

                assert!(con_iter.next_chunk(4).is_none());
                assert!(con_iter.next_chunk(4).is_none());

                let mut puller = con_iter.chunks_iter(5);
                assert!(puller.next().is_none());

                let mut iter = con_iter.chunks_iter(5).flattened();
                assert!(iter.next().is_none());
            });
        }
    });
}

#[test_matrix([Regular, Enumerated], [1, 2, 4])]
fn in_chunks<K: Enumeration>(_: K, nt: usize) {
    // let mut bag = ConcurrentBag::new();
    // let n = 123;
    // let vec: Vec<_> = (0..n).map(|x| (x + 10)).collect();
    // let slice = vec.as_slice();
    // let con_iter = ConIterSliceRef::<_, K>::new(slice);

    // std::thread::scope(|s| {
    //     for _ in 0..nt {
    //         s.spawn(|| {
    //             let mut puller = con_iter.in_chunks(5);
    //             while let Some(x) = puller.pull() {
    //                 let (begin_idx, iter) = K::destruct_next(x);
    //                 K::validate_begin_idx(begin_idx, |begin_idx| begin_idx % 5 == 0);
    //                 K::validate_begin_idx(begin_idx, |begin_idx| match begin_idx {
    //                     120 => iter.len() == 3,
    //                     _ => iter.len() == 5,
    //                 });
    //                 bag.push(12);

    //                 // for (i, x) in iter.enumerate() {
    //                 //     let idx = K::map_begin_idx(begin_idx, |begin_idx| begin_idx + i);
    //                 //     let item = K::construct_next(idx, x);
    //                 //     bag.push(item);
    //                 // }
    //             }
    //         });
    //     }
    // });

    // let mut puller = con_iter.in_chunks(5);
    // let mut i = 0;
    // while let Some(x) = puller.pull() {
    //     let (begin_idx, iter) = K::destruct_next(x);
    //     // assert!(K::eq_begin_idx(begin_idx, i));

    //     match i {
    //         120 => assert_eq!(iter.len(), 3),
    //         _ => assert_eq!(iter.len(), 5),
    //     };
    //     for x in iter {
    //         assert_eq!(*x, i + 10);
    //         i += 1;
    //     }
    // }
}
