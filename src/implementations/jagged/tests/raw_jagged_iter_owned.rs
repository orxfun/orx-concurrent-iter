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
