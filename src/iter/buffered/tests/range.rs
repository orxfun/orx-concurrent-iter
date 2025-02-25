use crate::{ConcurrentIter, IntoConcurrentIter};
use test_case::test_matrix;

#[test_matrix(
    [32, 1024],
    [2, 16],
    [1, 64]
)]
fn primitive(len: usize, num_threads: usize, batch: usize) {
    let source = 0..len;
    let expected_sum: i64 = source.clone().sum::<usize>() as i64;

    let con_iter = source.into_con_iter();
    let iter = &con_iter;

    let sum: i64 = std::thread::scope(|s| {
        let mut handles = alloc::vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut sum = 0i64;
                let mut chunks_iter = iter.buffered_iter(batch);
                while let Some(next) = chunks_iter.next() {
                    sum += next.values.sum::<usize>() as i64;
                }
                sum
            }));
        }

        handles.into_iter().map(|x| x.join().expect("-")).sum()
    });

    assert_eq!(sum, expected_sum);
}
