use super::indexers::{GeneralJaggedIndexer, MatrixIndexer};
use crate::implementations::jagged_arrays::owned::{
    into_iter::RawJaggedIterOwned, jagged_owned::RawJagged, raw_vec::RawVec,
    slice_iter::RawJaggedSliceIterOwned,
};
use test_case::test_matrix;

// matrix

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

#[test_matrix(
    [true, false],
    [true, false]
)]
fn raw_jagged_iter_owned_matrix2(consume_taken: bool, consume_remaining: bool) {
    let n = 4;
    let len = n * n;

    let jagged = || {
        let matrix = get_matrix(n);
        let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
        RawJagged::new(arrays, MatrixIndexer::new(n), Some(n * n))
    };

    for num_taken in 0..len {
        let mut jagged = jagged();
        unsafe { jagged.set_num_taken(Some(num_taken)) };

        {
            let iter_owned = RawJaggedSliceIterOwned::new(jagged.slice(0, num_taken));
            if consume_taken {
                let _from_jagged: Vec<_> = iter_owned.collect();
            }
        }

        let remaining = RawJaggedIterOwned::new(jagged);
        if consume_remaining {
            let remaining: Vec<_> = remaining.collect();
            let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
            assert_eq!(remaining, expected_remaining);
        }
    }
}

#[test]
fn abc() {
    let consume_taken = true;
    let consume_remaining = true;

    let n = 1;
    let len = n * n;

    let jagged = || {
        let matrix = get_matrix(n);
        let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
        RawJagged::new(arrays, MatrixIndexer::new(n), Some(n * n))
    };

    let num_taken = 1;
    {
        let mut jagged = jagged();
        unsafe { jagged.set_num_taken(Some(num_taken)) };

        {
            let iter_owned = RawJaggedSliceIterOwned::new(jagged.slice(0, num_taken));
            // if consume_taken {
            //     let _from_jagged: Vec<_> = iter_owned.collect();
            // }
        }

        // let remaining = RawJaggedIterOwned::new(jagged);
        // if consume_remaining {
        //     let remaining: Vec<_> = remaining.collect();
        //     let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
        //     assert_eq!(remaining, expected_remaining);
        // }
    }

    // for num_taken in 0..len {
    //     let mut jagged = jagged();
    //     unsafe { jagged.set_num_taken(Some(num_taken)) };

    //     {
    //         let iter_owned = RawJaggedSliceIterOwned::new(jagged.slice(0, num_taken));
    //         if consume_taken {
    //             let _from_jagged: Vec<_> = iter_owned.collect();
    //         }
    //     }

    //     let remaining = RawJaggedIterOwned::new(jagged);
    //     if consume_remaining {
    //         let remaining: Vec<_> = remaining.collect();
    //         let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
    //         assert_eq!(remaining, expected_remaining);
    //     }
    // }
}

// jagged

// fn get_jagged() -> Vec<Vec<String>> {
//     let jagged = vec![vec![0, 1], vec![2, 3, 4], vec![5], vec![6, 7, 8, 9]];
//     jagged
//         .into_iter()
//         .map(|x| x.into_iter().map(|x| x.to_string()).collect::<Vec<_>>())
//         .collect()
// }

// #[test_matrix(
//     [true, false],
//     [true, false]
// )]
// fn raw_jagged_iter_owned_jagged(consume_taken: bool, consume_remaining: bool) {
//     let len = 10;

//     let jagged = || {
//         let jagged = get_jagged();
//         RawJagged::new(jagged, GeneralJaggedIndexer, Some(10))
//     };

//     for num_taken in 0..len {
//         let mut jagged = jagged();
//         unsafe { jagged.set_num_taken(Some(num_taken)) };

//         {
//             let iter_owned = RawJaggedSliceIterOwned::new(jagged.slice(0, num_taken));
//             if consume_taken {
//                 let _from_jagged: Vec<_> = iter_owned.collect();
//             }
//         }

//         let remaining = RawJaggedIterOwned::new(jagged);
//         if consume_remaining {
//             let remaining: Vec<_> = remaining.collect();
//             let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
//             assert_eq!(remaining, expected_remaining);
//         }
//     }
// }
