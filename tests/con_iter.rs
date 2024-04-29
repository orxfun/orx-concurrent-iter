use orx_concurrent_bag::*;
use orx_concurrent_iter::*;
use std::{fmt::Debug, ops::Add};
use test_case::test_matrix;

fn concurrent_iter<I>(num_threads: usize, batch: usize, con_iter: I) -> Vec<I::Item>
where
    I: ConcurrentIter + Send + Sync,
    I::Item: Send + Sync + Clone + Copy + Debug + PartialEq,
{
    let collected = ConcurrentBag::new();
    let bag = &collected;

    let iter = &con_iter;

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                if batch == 1 {
                    while let Some(next) = iter.next_id_and_value() {
                        bag.push((next.idx, next.value));
                    }
                } else {
                    let mut more = true;
                    while more {
                        more = false;
                        let next = iter.next_chunk(batch);
                        let begin_idx = next.begin_idx();
                        for (i, value) in next.values().enumerate() {
                            bag.push((begin_idx + i, value));
                            more = true;
                        }
                    }
                }
            });
        }
    });

    let mut vec: Vec<_> = collected.into_inner().iter().copied().collect();
    vec.sort_by_key(|x| x.0);
    vec.iter().map(|x| x.1).collect()
}

fn concurrent_for_each_sum<I>(num_threads: usize, batch: usize, con_iter: I, expected_sum: usize)
where
    I: ConcurrentIter + Send + Sync,
    I::Item: Clone + Add<usize, Output = usize>,
    usize: Add<I::Item, Output = usize>,
{
    let iter = &con_iter;

    let sum: usize = std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| {
                s.spawn(move || {
                    let mut sum = 0;
                    if batch == 1 {
                        iter.for_each(|value| sum = sum + value);
                    } else {
                        iter.for_each_n(batch, |value| sum = sum + value);
                    }
                    sum
                })
            })
            .map(|x| x.join().unwrap())
            .sum()
    });
    assert_eq!(sum, expected_sum);
}

fn concurrent_enumerate_for_each_sum<I>(
    num_threads: usize,
    batch: usize,
    con_iter: I,
    expected_sum: usize,
    len: usize,
) where
    I: ConcurrentIter + Send + Sync,
    I::Item: Clone + Add<usize, Output = usize>,
    usize: Add<I::Item, Output = usize>,
{
    let iter = &con_iter;

    let sums = std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| {
                s.spawn(move || {
                    let mut sum = 0;
                    let mut sum_indices = 0;
                    if batch == 1 {
                        iter.enumerate_for_each(|idx, value| {
                            sum = sum + value;
                            sum_indices += idx;
                        });
                    } else {
                        iter.enumerate_for_each_n(batch, |idx, value| {
                            sum = sum + value;
                            sum_indices += idx;
                        });
                    }
                    (sum, sum_indices)
                })
            })
            .map(|x| x.join().unwrap())
            .collect::<Vec<_>>()
    });

    let sum: usize = sums.iter().map(|x| x.0).sum();
    let sum_indices: usize = sums.iter().map(|x| x.1).sum();

    assert_eq!(sum, expected_sum);
    assert_eq!(sum_indices, (0..len).sum::<usize>());
}

fn concurrent_sum<I>(num_threads: usize, batch: usize, con_iter: I, expected_sum: usize)
where
    I: ConcurrentIter + Send + Sync,
    I::Item: Clone + Add<usize, Output = usize>,
    usize: Add<I::Item, Output = usize>,
{
    let iter = &con_iter;

    let sum: usize = std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| {
                s.spawn(move || {
                    let mut sum = 0;
                    if batch == 1 {
                        while let Some(value) = iter.next() {
                            sum = sum + value;
                        }
                    } else {
                        let mut more = true;
                        while more {
                            more = false;
                            for value in iter.next_chunk(batch).values() {
                                sum = sum + value;
                                more = true;
                            }
                        }
                    }
                    sum
                })
            })
            .map(|x| x.join().unwrap())
            .sum()
    });

    assert_eq!(sum, expected_sum);
}

#[test_matrix(
    [1, 2, 4, 8, 64, 1024, 64*1024],
    [1, 2, 8],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn con_iter_slice(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();
    let sum: usize = source.iter().sum();

    let clone = source.clone();
    let slice = clone.as_slice();

    let collected: Vec<&usize> = concurrent_iter(num_threads, batch, slice.con_iter());
    assert_eq!(source, collected.into_iter().copied().collect::<Vec<_>>());
    assert_eq!(slice.try_get_exact_len(), Some(len));

    concurrent_sum(num_threads, batch, clone.as_slice().con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, clone.as_slice().con_iter(), sum);
    concurrent_enumerate_for_each_sum(num_threads, batch, clone.as_slice().con_iter(), sum, len);
}

#[test_matrix(
    [1, 2, 4, 8, 64, 1024, 64*1024],
    [1, 2, 8],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn con_iter_vec(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();
    let sum: usize = source.iter().sum();

    let collected: Vec<usize> = concurrent_iter(num_threads, batch, source.clone().into_con_iter());
    assert_eq!(source, collected.into_iter().collect::<Vec<_>>());
    assert_eq!(source.try_get_exact_len(), Some(len));

    concurrent_sum(num_threads, batch, source.clone().into_con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, source.clone().into_con_iter(), sum);
    concurrent_enumerate_for_each_sum(num_threads, batch, source.into_con_iter(), sum, len);
}

#[test_matrix(
    [1, 2, 4, 8, 64, 1024, 64*1024],
    [1, 2, 8],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn con_iter_iter(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();
    let sum: usize = source.iter().sum();

    let clone = source.clone();

    let collected: Vec<&usize> = concurrent_iter(num_threads, batch, clone.iter().into_con_iter());
    assert_eq!(source, collected.into_iter().copied().collect::<Vec<_>>());
    assert_eq!(source.try_get_exact_len(), Some(len));

    concurrent_sum(num_threads, batch, clone.iter().into_con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, clone.iter().into_con_iter(), sum);
    concurrent_enumerate_for_each_sum(num_threads, batch, clone.iter().into_con_iter(), sum, len);
}

#[test_matrix(
    [1, 2, 8],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn con_iter_array(num_threads: usize, batch: usize) {
    let mut source = [0usize; 1024];
    for (i, elem) in source.iter_mut().enumerate() {
        *elem = i;
    }
    let sum: usize = source.iter().sum();

    let collected: Vec<usize> = concurrent_iter(num_threads, batch, source.into_con_iter());
    assert_eq!(source.as_slice(), collected.into_iter().collect::<Vec<_>>());
    assert_eq!(source.try_get_exact_len(), Some(1024));

    concurrent_sum(num_threads, batch, source.into_con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, source.into_con_iter(), sum);
    concurrent_enumerate_for_each_sum(
        num_threads,
        batch,
        source.into_con_iter(),
        sum,
        source.len(),
    );
}
