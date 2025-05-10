use crate::implementations::jagged::{raw_jagged::RawJagged, raw_vec::RawVec};

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
fn raw_jagged_slice_iter_ref_matrix() {
    let n = 4;
    let matrix = get_matrix(n);

    let slices: Vec<_> = matrix.iter().map(|x| RawVec::from(x.as_slice())).collect();
    let jagged = RawJagged::new(slices, matrix_indexer(n), false);
    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }

    let vectors: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
    let jagged = RawJagged::new(vectors, matrix_indexer(n), true);
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

    let jagged = || {
        let slices: Vec<_> = matrix.iter().map(|x| RawVec::from(x.as_slice())).collect();
        RawJagged::new(slices, matrix_indexer(n), false)
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
fn raw_jagged_slice_iter_ref_jagged() {
    let data = get_jagged();
    let slices: Vec<_> = data.iter().map(|x| RawVec::from(x.as_slice())).collect();
    let jagged = RawJagged::new(slices, jagged_indexer(), false);
    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }

    let jagged = RawJagged::new(
        data.into_iter().map(RawVec::<String>::from).collect(),
        jagged_indexer(),
        true,
    );
    for begin in 0..jagged.len() {
        for end in begin..jagged.len() {
            let expected: Vec<_> = (begin..end).map(|x| x.to_string()).collect();
            let iter_ref = jagged.slice(begin, end).into_iter_ref();
            let from_jagged: Vec<_> = iter_ref.cloned().collect();

            assert_eq!(from_jagged, expected);
        }
    }
}
