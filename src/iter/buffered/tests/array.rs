use crate::*;
use test_case::test_matrix;

#[test_matrix(
    [2, 16],
    [1, 64]
)]
fn primitive(num_threads: usize, batch: usize) {
    let mut source = [0i64; 1024];
    for (i, x) in source.iter_mut().enumerate() {
        *x = i as i64;
    }
    let expected_sum: i64 = source.iter().sum();

    let con_iter = source.into_con_iter();
    let iter = &con_iter;

    let sum: i64 = std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut sum = 0i64;
                let mut chunks_iter = iter.buffered_iter(batch);
                while let Some(next) = chunks_iter.next() {
                    sum += next.values.sum::<i64>();
                }
                sum
            }));
        }

        handles.into_iter().map(|x| x.join().expect("-")).sum()
    });

    assert_eq!(sum, expected_sum);
}
