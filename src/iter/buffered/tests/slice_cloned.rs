use crate::*;
use alloc::{string::ToString, vec::Vec};
use test_case::test_matrix;

#[test_matrix(
    [32, 1024],
    [2, 16],
    [1, 64]
)]
fn primitive(len: usize, num_threads: usize, batch: usize) {
    let vec: Vec<_> = (0..len).map(|x| x as i64).collect();
    let source = vec.as_slice();
    let expected_sum: i64 = source.iter().sum();

    let con_iter = source.into_con_iter().cloned();
    let iter = &con_iter;

    let sum: i64 = std::thread::scope(|s| {
        let mut handles = alloc::vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut sum = 0i64;
                let mut chunks_iter = iter.buffered_iter_x(batch);
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

#[test_matrix(
    [32, 1024],
    [2, 16],
    [1, 64]
)]
fn heap(len: usize, num_threads: usize, batch: usize) {
    let vec: Vec<_> = (0..len).map(|x| x.to_string()).collect();
    let source = vec.as_slice();
    let expected_sum: usize = source.iter().map(|x| x.len()).sum();

    let con_iter = source.into_con_iter().cloned();
    let iter = &con_iter;

    let sum: usize = std::thread::scope(|s| {
        let mut handles = alloc::vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut sum = 0usize;
                let mut chunks_iter = iter.buffered_iter_x(batch);
                while let Some(next) = chunks_iter.next() {
                    sum += next.values.map(|x| x.len()).sum::<usize>();
                }
                sum
            }));
        }

        handles.into_iter().map(|x| x.join().expect("-")).sum()
    });

    assert_eq!(sum, expected_sum);
}
