use crate::implementations::jagged::{
    jagged_indexer::JaggedIndexer, raw_jagged::RawJagged, raw_vec::RawVec,
};

use super::indexers::{GeneralJaggedIndexer, MatrixIndexer};

// matrix

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

fn matrix_indexer(n: usize) -> impl Fn(usize) -> [usize; 2] + Clone {
    move |idx| {
        let f = idx / n;
        let i = idx % n;
        [f, i]
    }
}

#[test]
fn matrix_raw_jagged_drop_zero_taken() {
    let n = 4;
    let matrix = get_matrix(n);
    let _jagged = RawJagged::new_as_owned(
        matrix.into_iter().map(RawVec::<String>::from).collect(),
        MatrixIndexer::new(n),
        Some(n * n),
    );
}

#[test]
fn matrix_raw_jagged_drop_by_taken() {
    let n = 4;
    let len = n * n;

    for num_taken in 0..len {
        for num_iterated in 0..num_taken {
            let matrix = get_matrix(n);
            let mut jagged = RawJagged::new_as_owned(
                matrix.into_iter().map(RawVec::<String>::from).collect(),
                MatrixIndexer::new(n),
                Some(n * n),
            );
            unsafe { jagged.set_num_taken(Some(num_taken)) };

            let slice = jagged.slice(0, num_taken);
            let mut iter = slice.into_iter_owned();

            for i in 0..num_iterated {
                assert_eq!(iter.next(), Some(i.to_string()));
            }
        }
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

fn jagged_indexer() -> impl Fn(usize) -> [usize; 2] + Clone {
    let lengths = vec![2, 3, 1, 4];
    move |idx| match idx == lengths.iter().sum::<usize>() {
        true => [lengths.len() - 1, lengths[lengths.len() - 1]],
        false => {
            let mut idx = idx;
            let [mut f, mut i] = [0, 0];
            let mut current_f = 0;
            while idx > 0 {
                let current_len = lengths[current_f];
                match current_len > idx {
                    true => {
                        i = idx;
                        idx = 0;
                    }
                    false => {
                        f += 1;
                        idx -= current_len;
                    }
                }
                current_f += 1;
            }
            [f, i]
        }
    }
}

#[test]
fn jagged_raw_jagged_drop_zero_taken() {
    let jagged = get_jagged();
    let _jagged = RawJagged::new_as_owned(
        jagged.into_iter().map(RawVec::<String>::from).collect(),
        GeneralJaggedIndexer::new(10),
        Some(10),
    );
}

#[test]
fn jagged_raw_jagged_drop_by_taken() {
    let len = 10;
    for num_taken in 0..len {
        for num_iterated in 0..num_taken {
            let jagged = get_jagged();
            let mut jagged = RawJagged::new_as_owned(
                jagged.into_iter().map(RawVec::<String>::from).collect(),
                GeneralJaggedIndexer::new(10),
                Some(10),
            );
            unsafe { jagged.set_num_taken(Some(num_taken)) };

            let slice = jagged.slice(0, num_taken);
            let mut iter = slice.into_iter_owned();

            for i in 0..num_iterated {
                assert_eq!(iter.next(), Some(i.to_string()));
            }
        }
    }
}
