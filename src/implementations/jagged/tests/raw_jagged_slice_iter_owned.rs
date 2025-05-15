use super::indexers::{GeneralJaggedIndexer, MatrixIndexer};
use crate::implementations::jagged::{raw_jagged::RawJagged, raw_vec::RawVec};

// matrix

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
    let len = n * n;
    let indexer = || MatrixIndexer::new(n);

    let jagged = || {
        let matrix = get_matrix(n);
        let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
        RawJagged::new_as_owned(arrays, indexer(), Some(n * n))
    };

    for num_taken in 0..len {
        let mut jagged = jagged();
        unsafe { jagged.set_num_taken(Some(num_taken)) };

        let expected: Vec<_> = (0..num_taken).map(|x| x.to_string()).collect();
        let iter_owned = jagged.slice(0, num_taken).into_iter_owned();
        let from_jagged: Vec<_> = iter_owned.collect();

        assert_eq!(from_jagged, expected);
    }
}

#[test]
fn raw_jagged_slice_iter_owned_matrix_twice_iteration() {
    let n = 4;
    let len = n * n;
    let indexer = || MatrixIndexer::new(n);

    let jagged = || {
        let matrix = get_matrix(n);
        let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
        RawJagged::new_as_owned(arrays, indexer(), Some(n * n))
    };

    for num_taken_1 in 0..len {
        for num_taken_2 in num_taken_1..len {
            let mut jagged = jagged();
            unsafe { jagged.set_num_taken(Some(num_taken_2)) };

            let expected: Vec<_> = (0..num_taken_1).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(0, num_taken_1).into_iter_owned();
            let from_jagged: Vec<_> = iter_ref.collect();

            assert_eq!(from_jagged, expected);

            let expected: Vec<_> = (num_taken_1..num_taken_2).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(num_taken_1, num_taken_2).into_iter_owned();
            let from_jagged: Vec<_> = iter_ref.collect();

            assert_eq!(from_jagged, expected);
        }
    }
}

#[test]
fn raw_jagged_slice_iter_owned_exact_size_matrix() {
    let n = 4;
    let len = n * n;
    let indexer = || MatrixIndexer::new(n);

    let jagged = || {
        let matrix = get_matrix(n);
        let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
        RawJagged::new_as_owned(arrays, indexer(), Some(n * n))
    };

    for num_taken in 0..len {
        let mut jagged = jagged();
        unsafe { jagged.set_num_taken(Some(num_taken)) };

        let mut iter_owned = jagged.slice(0, num_taken).into_iter_owned();

        let mut len = num_taken;
        assert_eq!(iter_owned.len(), len);
        while let Some(_) = iter_owned.next() {
            len -= 1;
            assert_eq!(iter_owned.len(), len);
        }

        assert_eq!(iter_owned.len(), 0);
        assert_eq!(iter_owned.next(), None);
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
fn raw_jagged_slice_iter_owned_jagged() {
    let len = 10;
    let indexer = || GeneralJaggedIndexer::new(len);

    let jagged = || {
        let data = get_jagged();
        let arrays: Vec<_> = data.into_iter().map(RawVec::from).collect();
        RawJagged::new_as_owned(arrays, indexer(), Some(len))
    };

    for num_taken in 0..len {
        let mut jagged = jagged();
        unsafe { jagged.set_num_taken(Some(num_taken)) };

        let expected: Vec<_> = (0..num_taken).map(|x| x.to_string()).collect();
        let iter_ref = jagged.slice(0, num_taken).into_iter_owned();
        let from_jagged: Vec<_> = iter_ref.collect();

        assert_eq!(from_jagged, expected);
    }
}

#[test]
fn raw_jagged_slice_iter_owned_jagged_twice_iteration() {
    let len = 10;
    let indexer = || GeneralJaggedIndexer::new(len);

    let jagged = || {
        let data = get_jagged();
        let arrays: Vec<_> = data.into_iter().map(RawVec::from).collect();
        RawJagged::new_as_owned(arrays, indexer(), Some(len))
    };

    for num_taken_1 in 0..len {
        for num_taken_2 in num_taken_1..len {
            let mut jagged = jagged();
            unsafe { jagged.set_num_taken(Some(num_taken_2)) };

            let expected: Vec<_> = (0..num_taken_1).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(0, num_taken_1).into_iter_owned();
            let from_jagged: Vec<_> = iter_ref.collect();

            assert_eq!(from_jagged, expected);

            let expected: Vec<_> = (num_taken_1..num_taken_2).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(num_taken_1, num_taken_2).into_iter_owned();
            let from_jagged: Vec<_> = iter_ref.collect();

            assert_eq!(from_jagged, expected);
        }
    }
}
