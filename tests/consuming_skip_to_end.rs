use orx_concurrent_iter::*;
use std::{
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
};
use test_case::test_matrix;

fn run<T, C>(iter: &C, num_threads: usize, batch: usize)
where
    T: Send + Sync,
    C: ConcurrentIter<Item = T>,
{
    let len = iter.try_get_len().expect("is-some");
    let until = len / 2;
    let counter = &AtomicUsize::new(0);

    std::thread::scope(|s| {
        for t in 0..num_threads {
            s.spawn(move || match (batch, t % 2 == 0) {
                (1, _) => {
                    while let Some(next) = iter.next_id_and_value() {
                        let idx = counter.fetch_add(1, Ordering::Relaxed);
                        if idx > until {
                            iter.skip_to_end();
                            break;
                        }

                        let _idx = black_box(next.idx);
                        let _value = black_box(next.value);
                    }
                }
                (_, true) => {
                    while let Some(chunk) = iter.next_chunk(batch) {
                        let idx = counter.fetch_add(chunk.values.len(), Ordering::Relaxed);
                        if idx > until {
                            iter.skip_to_end();
                            break;
                        }

                        for value in chunk.values {
                            let _value = black_box(value);
                        }
                    }
                }
                (_, false) => {
                    let mut buffer = iter.buffered_iter_x(batch);
                    while let Some(chunk) = buffer.next_x() {
                        let idx = counter.fetch_add(chunk.len(), Ordering::Relaxed);
                        if idx > until {
                            iter.skip_to_end();
                            break;
                        }

                        for value in chunk {
                            let _value = black_box(value);
                        }
                    }
                }
            });
        }
    });
}

#[test_matrix(
    [1, 4, 1024, 2141, 2*1024, 4*1024+1],
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn vec_string_skip_to_end(len: usize, num_threads: usize, batch: usize) {
    let vec: Vec<_> = (0..len).map(|i| i.to_string()).collect();
    run(&vec.into_con_iter(), num_threads, batch);
}

#[test_matrix(
    [1, 4, 1024, 2141, 2*1024, 4*1024+1],
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn vec_usize_skip_to_end(len: usize, num_threads: usize, batch: usize) {
    let vec: Vec<_> = (0..len).map(|i| i as usize).collect();
    run(&vec.into_con_iter(), num_threads, batch);
}

#[test_matrix(
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn array_usize_skip_to_end(num_threads: usize, batch: usize) {
    let mut array = [0i64; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i as i64;
    }
    run(&array.into_con_iter(), num_threads, batch);
}
