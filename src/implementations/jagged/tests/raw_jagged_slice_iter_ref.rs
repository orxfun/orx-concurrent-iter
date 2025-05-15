use super::indexers::{GeneralJaggedIndexer, MatrixIndexer};
use crate::implementations::{jagged::raw_jagged::RawJagged, jagged::raw_vec::RawVec};

// matrix

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

#[test]
fn raw_jagged_slice_iter_ref_matrix() {
    let n = 4;
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(n * n));

    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }

    let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
    let jagged = RawJagged::new_as_owned(arrays, indexer(), Some(n * n));

    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }
}

#[test]
fn raw_jagged_slice_iter_ref_exact_size_matrix() {
    let n = 4;
    let len = n * n;
    let matrix = get_matrix(n);
    let indexer = || MatrixIndexer::new(n);

    let jagged = || {
        let slices: Vec<_> = matrix.iter().map(|v| v.as_slice().into()).collect();
        RawJagged::new_as_reference(slices, indexer(), Some(n * n))
    };

    for num_taken in 0..len {
        let jagged = jagged();

        let mut iter_ref = jagged.slice(0, num_taken).into_iter_ref();

        let mut len = num_taken;
        assert_eq!(iter_ref.len(), len);
        while let Some(_) = iter_ref.next() {
            len -= 1;
            assert_eq!(iter_ref.len(), len);
        }

        assert_eq!(iter_ref.len(), 0);
        assert_eq!(iter_ref.next(), None);
    }
}

// jagged

fn get_jagged() -> Vec<Vec<String>> {
    let jagged = vec![vec![0, 1], vec![2, 3, 4], vec![5], vec![6, 7, 8, 9]];
    jagged
        .into_iter()
        .map(|x| x.into_iter().map(|x| x.to_string()).collect::<Vec<_>>())
        .collect()
}

#[test]
fn raw_jagged_slice_iter_ref_jagged() {
    let data = get_jagged();
    let len = data.iter().map(|x| x.len()).sum();
    let indexer = || GeneralJaggedIndexer;
    let slices: Vec<_> = data.iter().map(|v| v.as_slice().into()).collect();
    let jagged = RawJagged::new_as_reference(slices, indexer(), Some(len));

    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }

    let arrays: Vec<_> = data.into_iter().map(RawVec::from).collect();
    let jagged = RawJagged::new_as_owned(arrays, indexer(), Some(len));

    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }
}
