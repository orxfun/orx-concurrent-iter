use crate::implementations::jagged::{jagged_index::JaggedIndex, raw_jagged::RawJagged};

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

#[test]
fn raw_jagged_indices() {
    let n = 4;
    let matrix = get_matrix(n);

    let iter = (0..n).map(|i| matrix[i].as_slice());
    let indexer = |idx| {
        let f = idx / n;
        let i = idx % n;
        [f, i]
    };

    let jagged = RawJagged::new(iter, indexer);
    assert_eq!(jagged.len(), n * n);

    for flat_index in 0..jagged.len() {
        let f = flat_index / n;
        let i = flat_index % n;

        let idx = jagged.jagged_index(flat_index);
        assert_eq!(idx, Some(JaggedIndex::new(f, i)));
    }

    let idx = jagged.jagged_index(jagged.len());
    assert_eq!(idx, Some(JaggedIndex::new(n - 1, n)));
}

#[test]
fn raw_jagged_slices() {
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
            let mut from_jagged = Vec::new();
            let jagged_slice = jagged.slice(begin, end);
            for s in 0..jagged_slice.num_slices() {
                from_jagged.extend(jagged_slice.get_slice(s).unwrap().iter().cloned());
            }
            assert_eq!(from_jagged, expected);
        }
    }
}
