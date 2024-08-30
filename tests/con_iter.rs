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
        for t in 0..num_threads {
            s.spawn(move || {
                if batch == 1 {
                    while let Some(next) = iter.next_id_and_value() {
                        bag.push((next.idx, next.value));
                    }
                }
                if t % 2 == 0 {
                    while let Some(chunk) = iter.next_chunk(batch) {
                        for (i, value) in chunk.values.enumerate() {
                            bag.push((chunk.begin_idx + i, value));
                        }
                    }
                } else {
                    let mut buffered_iter = iter.buffered_iter(batch);
                    while let Some(chunk) = buffered_iter.next() {
                        for (i, value) in chunk.values.enumerate() {
                            bag.push((chunk.begin_idx + i, value));
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

fn concurrent_iter_exact_len<I: ConcurrentIter>(iter: I, expected_len: usize, batch: usize) {
    assert_eq!(iter.try_get_len(), Some(expected_len));
    let mut remaining = expected_len;

    match batch {
        1 => {
            while let Some(_) = iter.next_id_and_value() {
                remaining -= 1;
                assert_eq!(iter.try_get_len(), Some(remaining));
            }
            assert_eq!(iter.try_get_len(), Some(0));

            _ = iter.next_id_and_value();
            assert_eq!(iter.try_get_len(), Some(0));
        }
        x if x % 2 == 0 => {
            while let Some(chunk) = iter.next_chunk(x) {
                let chunk_len = chunk.values.len();
                remaining -= chunk_len;
                assert_eq!(iter.try_get_len(), Some(remaining));
            }
            assert_eq!(iter.try_get_len(), Some(0));

            _ = iter.next_chunk(x);
            assert_eq!(iter.try_get_len(), Some(0));
        }
        x => {
            let mut buffered_iter = iter.buffered_iter(x);
            while let Some(chunk) = buffered_iter.next() {
                let chunk_len = chunk.values.len();
                remaining -= chunk_len;
                assert_eq!(iter.try_get_len(), Some(remaining));
            }
            assert_eq!(iter.try_get_len(), Some(0));

            _ = buffered_iter.next();
            assert_eq!(iter.try_get_len(), Some(0));
        }
    }
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
                    iter.for_each(batch, |value| sum = sum + value);
                    sum
                })
            })
            .map(|x| x.join().unwrap())
            .sum()
    });
    assert_eq!(sum, expected_sum);
}

fn concurrent_fold_sum<I>(num_threads: usize, batch: usize, con_iter: I, expected_sum: usize)
where
    I: ConcurrentIter + Send + Sync,
    I::Item: Clone + Add<usize, Output = usize>,
    usize: Add<I::Item, Output = usize>,
{
    let iter = &con_iter;

    let sum: usize = std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| s.spawn(move || iter.fold(batch, 0usize, |x, y| x + y)))
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
                    iter.enumerate_for_each(batch, |idx, value| {
                        sum = sum + value;
                        sum_indices += idx;
                    });
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
                        while let Some(chunk) = iter.next_chunk(batch) {
                            for value in chunk.values {
                                sum = sum + value;
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
    [1, 4, 1024, 64*1024],
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_slice(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();
    let sum: usize = source.iter().sum();

    let clone = source.clone();
    let slice = clone.as_slice();

    let collected: Vec<&usize> = concurrent_iter(num_threads, batch, slice.con_iter());
    assert_eq!(source, collected.into_iter().copied().collect::<Vec<_>>());

    concurrent_sum(num_threads, batch, clone.as_slice().con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, clone.as_slice().con_iter(), sum);
    concurrent_fold_sum(num_threads, batch, clone.as_slice().con_iter(), sum);
    concurrent_enumerate_for_each_sum(num_threads, batch, clone.as_slice().con_iter(), sum, len);
    concurrent_iter_exact_len(clone.clone().as_slice().con_iter(), len, 1);
    concurrent_iter_exact_len(clone.clone().as_slice().con_iter(), len, 32);
    concurrent_iter_exact_len(clone.clone().as_slice().con_iter(), len, 33);
}

#[test_matrix(
    [1, 4, 1024, 64*1024],
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_vec(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();
    let sum: usize = source.iter().sum();

    let collected: Vec<usize> = concurrent_iter(num_threads, batch, source.clone().into_con_iter());
    assert_eq!(source, collected.into_iter().collect::<Vec<_>>());

    concurrent_sum(num_threads, batch, source.clone().into_con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, source.clone().into_con_iter(), sum);
    concurrent_fold_sum(num_threads, batch, source.clone().into_con_iter(), sum);
    concurrent_enumerate_for_each_sum(num_threads, batch, source.clone().into_con_iter(), sum, len);
    concurrent_iter_exact_len(source.clone().into_con_iter(), len, 1);
    concurrent_iter_exact_len(source.clone().into_con_iter(), len, 32);
    concurrent_iter_exact_len(source.clone().into_con_iter(), len, 33);
}

#[test_matrix(
    [1, 4, 1024, 64*1024],
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_iter(len: usize, num_threads: usize, batch: usize) {
    let source: Vec<_> = (0..len).collect();
    let sum: usize = source.iter().sum();

    let clone = source.clone();

    let collected: Vec<&usize> = concurrent_iter(num_threads, batch, clone.iter().into_con_iter());
    assert_eq!(source, collected.into_iter().copied().collect::<Vec<_>>());

    concurrent_sum(num_threads, batch, clone.iter().into_con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, clone.iter().into_con_iter(), sum);
    concurrent_fold_sum(num_threads, batch, clone.iter().into_con_iter(), sum);
    concurrent_enumerate_for_each_sum(num_threads, batch, clone.iter().into_con_iter(), sum, len);
    concurrent_iter_exact_len(clone.iter().into_con_iter(), len, 1);
    concurrent_iter_exact_len(clone.iter().into_con_iter(), len, 32);
    concurrent_iter_exact_len(clone.iter().into_con_iter(), len, 33);
    concurrent_iter_exact_len(clone.iter().map(|x| x * 2).into_con_iter(), len, 1);
    concurrent_iter_exact_len(clone.iter().map(|x| x * 2).into_con_iter(), len, 32);
    concurrent_iter_exact_len(clone.iter().map(|x| x * 2).into_con_iter(), len, 33);
}

#[test_matrix(
    [1, 2, 8],
    [1, 4, 64, 1024]
)]
fn con_iter_array(num_threads: usize, batch: usize) {
    let mut source = [0usize; 1024];
    for (i, elem) in source.iter_mut().enumerate() {
        *elem = i;
    }
    let sum: usize = source.iter().sum();

    let collected: Vec<usize> = concurrent_iter(num_threads, batch, source.con_iter().cloned());
    assert_eq!(source.as_slice(), collected.into_iter().collect::<Vec<_>>());

    concurrent_sum(num_threads, batch, source.into_con_iter(), sum);
    concurrent_for_each_sum(num_threads, batch, source.into_con_iter(), sum);
    concurrent_fold_sum(num_threads, batch, source.into_con_iter(), sum);
    concurrent_enumerate_for_each_sum(
        num_threads,
        batch,
        source.into_con_iter(),
        sum,
        source.len(),
    );
    concurrent_iter_exact_len(source.into_con_iter(), 1024, 1);
    concurrent_iter_exact_len(source.into_con_iter(), 1024, 32);
    concurrent_iter_exact_len(source.into_con_iter(), 1024, 33);
}
