use test_case::test_matrix;

use crate::implementations::jagged::{raw_jagged::RawJagged, raw_vec::RawVec};

// matrix

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

fn matrix_indexer(n: usize) -> impl Fn(usize) -> [usize; 2] {
    move |idx| {
        let f = idx / n;
        let i = idx % n;
        [f, i]
    }
}

#[test_matrix(
    [true, false],
    [true, false]
)]
fn raw_jagged_iter_owned_matrix(consume_taken: bool, consume_remaining: bool) {
    let n = 4;
    let len = n * n;

    let jagged = || {
        let matrix = get_matrix(n);
        let vectors: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
        RawJagged::new(vectors.into_iter(), matrix_indexer(n), true)
    };

    for num_taken in 0..len {
        let mut jagged = jagged();
        jagged.set_num_taken(num_taken);

        {
            let iter_owned = jagged.slice(0, num_taken).into_iter_owned();
            if consume_taken {
                let _from_jagged: Vec<_> = iter_owned.collect();
            }
        }

        let remaining = jagged.into_iter_owned();
        if consume_remaining {
            let remaining: Vec<_> = remaining.collect();
            let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
            assert_eq!(remaining, expected_remaining);
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

fn jagged_indexer() -> impl Fn(usize) -> [usize; 2] {
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

#[test_matrix(
    [true, false],
    [true, false]
)]
fn raw_jagged_iter_owned_jagged(consume_taken: bool, consume_remaining: bool) {
    let len = 10;

    let jagged = || {
        let jagged = get_jagged();
        let vectors: Vec<_> = jagged.into_iter().map(RawVec::from).collect();
        RawJagged::new(vectors.into_iter(), jagged_indexer(), true)
    };

    for num_taken in 0..len {
        let mut jagged = jagged();
        jagged.set_num_taken(num_taken);

        {
            let iter_owned = jagged.slice(0, num_taken).into_iter_owned();
            if consume_taken {
                let _from_jagged: Vec<_> = iter_owned.collect();
            }
        }

        let remaining = jagged.into_iter_owned();
        if consume_remaining {
            let remaining: Vec<_> = remaining.collect();
            let expected_remaining: Vec<_> = (num_taken..len).map(|x| x.to_string()).collect();
            assert_eq!(remaining, expected_remaining);
        }
    }
}
