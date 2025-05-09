use crate::implementations::jagged::raw_jagged::RawJagged;
use orx_iterable::Collection;

#[test]
fn raw_jagged_from_matrix() {
    let n = 4;
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(
            ((i * n)..((i + 1) * n))
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
        );
    }

    let iter = (0..n).map(|i| matrix[i].as_slice());
    let indexer = |idx| {
        let f = idx / n;
        let i = idx % n;
        [f, i]
    };

    let jagged = RawJagged::new(iter, indexer);

    assert_eq!(jagged.num_slices(), n);
    assert_eq!(jagged.len(), n * n);

    for (a, b) in jagged.iter().zip(matrix.iter()) {
        assert_eq!(a.slice_from(0), Some(b.as_slice()));
    }

    // TODO: test the slice method
}
