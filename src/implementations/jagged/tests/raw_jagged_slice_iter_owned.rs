use crate::implementations::jagged::raw_jagged::RawJagged;

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

#[test]
fn raw_jagged_slice_iter_owned_matrix() {
    let n = 4;
    let matrix = get_matrix(n);

    let iter = (0..n).map(|i| matrix[i].as_slice());
    let indexer = |idx| {
        let f = idx / n;
        let i = idx % n;
        [f, i]
    };

    let jagged = RawJagged::new(iter, indexer);

    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_owned = jagged.slice(begin, end).into_iter_owned();
            let from_jagged: Vec<_> = iter_owned.collect();

            assert_eq!(from_jagged, expected);
        }
    }
}

// #[test]
// fn raw_jagged_slice_iter_owned_jagged() {
//     let matrix = vec![vec![0, 1], vec![2, 3, 4], vec![5], vec![6, 7, 8, 9]];
//     let matrix: Vec<_> = matrix
//         .into_iter()
//         .map(|x| x.into_iter().map(|x| x.to_string()).collect::<Vec<_>>())
//         .collect();

//     let iter = (0..matrix.len()).map(|i| matrix[i].as_slice());
//     let lengths = vec![2, 3, 1, 4];
//     let indexer = |idx| match idx == lengths.iter().sum() {
//         true => [lengths.len() - 1, lengths[lengths.len() - 1]],
//         false => {
//             let mut idx = idx;
//             let [mut f, mut i] = [0, 0];
//             let mut current_f = 0;
//             while idx > 0 {
//                 let current_len = lengths[current_f];
//                 match current_len > idx {
//                     true => {
//                         i = idx;
//                         idx = 0;
//                     }
//                     false => {
//                         f += 1;
//                         idx -= current_len;
//                     }
//                 }
//                 current_f += 1;
//             }
//             [f, i]
//         }
//     };

//     let jagged = RawJagged::new(iter, indexer);

//     for begin in 0..jagged.len() {
//         for end in begin..jagged.len() {
//             let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
//             let iter_ref = jagged.slice(begin, end).into_iter_ref();
//             let from_jagged: Vec<_> = iter_ref.cloned().collect();

//             assert_eq!(from_jagged, expected);
//         }
//     }
// }
