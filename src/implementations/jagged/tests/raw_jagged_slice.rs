use super::indexers::MatrixIndexer;
use crate::implementations::jagged::{
    jagged_index::JaggedIndex, raw_jagged::RawJagged, raw_jagged_slice::RawJaggedSlice,
    raw_vec::RawVec,
};
use test_case::test_matrix;

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(((i * n)..((i + 1) * n)).map(|x| x.to_string()).collect());
    }
    matrix
}

fn assert_empty(slice: &RawJaggedSlice<String>, num_test_slice_index: usize) {
    assert_eq!(slice.num_slices(), 0);
    for i in 0..num_test_slice_index {
        assert_eq!(slice.get_slice(i), None);
    }
}

fn into_jagged_index(idx: [usize; 2]) -> JaggedIndex {
    JaggedIndex::new(idx[0], idx[1])
}

#[should_panic]
#[test_matrix([
    ([0, 1], [0, 0]),
    ([1, 0], [0, 0]),
    ([1, 2], [1, 1]),
    ([2, 1], [1, 1]),
    ([3, 5], [3, 5]),
    ([3, 1], [3, 5]),
    ([4, 0], [4, 1]),
    ([3, 0], [4, 1]),
    ([5, 0], [5, 0]),
    ([4, 0], [5, 0]),
])]
fn invalid_raw_jagged_slice_indices((begin, end): ([usize; 2], [usize; 2])) {
    let [begin, end] = [begin, end].map(into_jagged_index);
    let n = 4;
    let x = MatrixIndexer::new(n);

    let matrix = get_matrix(n);
    let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
    let slice_len = x.slice_len(&arrays, &begin, &end);

    let jagged = RawJagged::new_as_owned(arrays, x, Some(n * n));
    let _slice = RawJaggedSlice::new(jagged.arrays(), begin, end, slice_len);
}

#[test]
fn default_raw_jagged_slice() {
    let empty_slice = RawJaggedSlice::<String>::default();
    assert_empty(&empty_slice, 10);
}

#[test]
fn empty_non_default_raw_jagged_slice() {
    let n = 4;
    let x = MatrixIndexer::new(n);

    let matrix = get_matrix(n);
    let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();

    let jagged = RawJagged::new_as_owned(arrays, x.clone(), Some(n * n));

    let empty_indices = [
        ([0, 0], [0, 0]),
        ([1, 0], [1, 0]),
        ([3, 3], [3, 3]),
        ([2, 2], [2, 2]),
        ([2, 3], [2, 3]),
    ];

    for (begin, end) in empty_indices {
        let [begin, end] = [begin, end].map(into_jagged_index);
        let slice_len = x.slice_len(jagged.arrays(), &begin, &end);
        let empty_slice = RawJaggedSlice::new(jagged.arrays(), begin, end, slice_len);
        assert_empty(&empty_slice, 10);
    }
}

#[test]
fn non_empty_raw_jagged_slice() {
    let n = 4;
    let len = n * n;

    for begin in 0..len {
        for end in begin..len {
            validate_raw_jagged_slice(begin, end);
        }
    }
}

fn validate_raw_jagged_slice(flat_begin: usize, flat_end: usize) {
    let n = 4;
    let x = MatrixIndexer::new(n);

    let matrix = get_matrix(n);
    let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();

    let jagged = RawJagged::new_as_owned(arrays, x, Some(n * n));

    let [f, i] = [flat_begin / n, flat_begin % n];
    let begin = JaggedIndex::new(f, i);
    let [f, i] = [flat_end / n, flat_end % n];
    let end = JaggedIndex::new(f, i);

    let slice = RawJaggedSlice::new(
        jagged.arrays(),
        begin.clone(),
        end.clone(),
        flat_end - flat_begin,
    );
    let expected: Vec<_> = (flat_begin..flat_end).map(|x| x.to_string()).collect();
    let mut slice_from_jagged = Vec::new();
    for s in 0..slice.num_slices() {
        slice_from_jagged.extend(slice.get_slice(s).unwrap().iter().cloned());
    }
    assert_eq!(slice_from_jagged, expected);
}

#[test]
fn non_empty_raw_jagged_slice_from() {
    let n = 4;
    let len = n * n;

    for begin in 0..len {
        validate_raw_jagged_slice_from(begin);
    }
}

fn validate_raw_jagged_slice_from(flat_begin: usize) {
    let n = 4;
    let x = MatrixIndexer::new(n);

    let matrix = get_matrix(n);
    let arrays: Vec<_> = matrix.into_iter().map(RawVec::from).collect();

    let jagged = RawJagged::new_as_owned(arrays, x, Some(n * n));

    let slice = jagged.slice_from(flat_begin);
    let expected: Vec<_> = (flat_begin..(n * n)).map(|x| x.to_string()).collect();
    let mut slice_from_jagged = Vec::new();
    for s in 0..slice.num_slices() {
        slice_from_jagged.extend(slice.get_slice(s).unwrap().iter().cloned());
    }
    assert_eq!(slice_from_jagged, expected);
}
