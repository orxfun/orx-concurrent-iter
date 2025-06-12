use crate::{
    ConcurrentIter, ExactSizeConcurrentIter, implementations::vec_drain::con_iter::ConIterVecDrain,
};
use core::ops::RangeBounds;
use test_case::test_matrix;

#[test_matrix(
    [30],
    [.., 5.., ..25, 5..25],
    [0, 1, 15, 29, 30, 35]
)]
fn validation(n: usize, range: impl RangeBounds<usize> + Clone, num_pull: usize) {
    let mut vec1: Vec<_> = (0..n).map(|x| x.to_string()).collect();
    let mut vec2 = vec1.clone();

    {
        let mut iter = vec1.drain(range.clone());
        let con_iter = ConIterVecDrain::new(&mut vec2, range);

        for _ in 0..num_pull {
            let a = iter.next();
            let b = con_iter.next();
            assert_eq!(a, b);
            assert_eq!(iter.len(), con_iter.len());
        }
    }

    assert_eq!(vec1, vec2);
}
