// use crate::implementations::jagged::raw_jagged::RawJagged;
// use orx_iterable::{Collection, Iterable};

// fn get_matrix(n: usize) -> Vec<Vec<String>> {
//     let mut matrix = Vec::new();
//     for i in 0..n {
//         matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
//     }
//     matrix
// }

// #[test]
// fn raw_jagged_from_matrix() {
//     let n = 4;
//     let matrix = get_matrix(n);

//     let iter = (0..n).map(|i| matrix[i].as_slice());
//     let indexer = |idx| {
//         let f = idx / n;
//         let i = idx % n;
//         [f, i]
//     };

//     let jagged = RawJagged::new(iter, indexer);

//     assert_eq!(jagged.num_slices(), n);
//     assert_eq!(jagged.len(), n * n);

//     for (a, b) in jagged.iter().zip(matrix.iter()) {
//         assert_eq!(a.slice_from(0), Some(b.as_slice()));
//     }

//     let len = n * n;
//     for i in 0..len {
//         for j in (i + 1)..len {
//             let expected: Vec<_> = (0..j).map(|x| x.to_string()).collect();

//             let mut from_slice = Vec::new();
//             let slice = jagged.slice(i, j - 1);
//             dbg!(i, j, slice.num_slices());
//             for s in 0..slice.num_slices() {
//                 let slice = slice.get_slice(s).unwrap();
//                 from_slice.extend(slice.cloned().iter());
//             }

//             assert_eq!(from_slice, expected);
//         }
//     }
// }
