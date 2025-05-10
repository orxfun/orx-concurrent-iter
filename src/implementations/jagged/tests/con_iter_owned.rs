use crate::{
    concurrent_iter::ConcurrentIter,
    exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::jagged::{
        con_iter_owned::ConIterJaggedOwned, raw_jagged::RawJagged, raw_vec::RawVec,
    },
};
use orx_concurrent_bag::ConcurrentBag;
use test_case::test_matrix;

#[cfg(miri)]
const N: usize = 125;
#[cfg(not(miri))]
const N: usize = 4735;

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
fn abc_enumeration() {
    let n = 1;
    let matrix = get_matrix(n);
    let vectors: Vec<_> = matrix.into_iter().map(RawVec::from).collect();
    let jagged = RawJagged::new(vectors.into_iter(), matrix_indexer(n), true);
    let iter = ConIterJaggedOwned::new(jagged, 0);

    assert_eq!(iter.next(), Some(0.to_string()));
    // assert_eq!(iter.next_with_idx(), Some((1, 1.to_string())));
    // assert_eq!(iter.next(), Some(2.to_string()));
    // assert_eq!(iter.next_with_idx(), Some((3, 3.to_string())));
    // assert_eq!(iter.next(), Some(4.to_string()));
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next_with_idx(), None);
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next_with_idx(), None);
}
