use crate::implementations::jagged::{raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};
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

#[test]
fn default_raw_jagged_slice() {
    let empty_slice = RawJaggedSlice::<String>::default();
    assert_empty(&empty_slice, 10);
}

#[test]
fn empty_non_default_raw_jagged_slice() {
    let n = 4;
    let matrix = get_matrix(n);
    let slices: Vec<_> = (0..n)
        .map(|i| matrix[i].as_slice())
        .map(RawSlice::from)
        .collect();

    let empty_slice = RawJaggedSlice::new(&slices, [0, 0], [0, 0]);
    assert_empty(&empty_slice, 10);

    let empty_slice = RawJaggedSlice::new(&slices, [1, 0], [1, 0]);
    assert_empty(&empty_slice, 10);

    let empty_slice = RawJaggedSlice::new(&slices, [4, 0], [4, 0]);
    assert_empty(&empty_slice, 10);

    let empty_slice = RawJaggedSlice::new(&slices, [2, 2], [2, 2]);
    assert_empty(&empty_slice, 10);
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
    let n = 4;
    let matrix = get_matrix(n);
    let slices: Vec<_> = (0..n)
        .map(|i| matrix[i].as_slice())
        .map(RawSlice::from)
        .collect();
    let _slice = RawJaggedSlice::new(&slices, begin, end);
}
