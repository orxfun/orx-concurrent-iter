use orx_concurrent_iter::*;
use std::{
    hint::black_box,
    sync::atomic::{AtomicUsize, Ordering},
};
use test_case::test_matrix;

#[cfg(not(miri))]
const LEN: [usize; 6] = [1, 4, 1024, 2141, 2 * 1024, 4 * 1024 + 1];
#[cfg(miri)]
const LEN: [usize; 6] = [1, 4, 32, 33, 2 * 64, 4 * 64 + 1];

fn run<T, C>(iter: &C, num_threads: usize, batch: usize)
where
    T: Send + Sync,
    C: ConcurrentIter<Item = T>,
{
    let len = iter.try_get_len().expect("is-some");
    let until = len / 2;
    let counter = &AtomicUsize::new(0);

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || match batch {
                1 => {
                    while let Some((idx, value)) = iter.next_with_idx() {
                        let atomic_idx = counter.fetch_add(1, Ordering::Relaxed);
                        if atomic_idx > until {
                            iter.skip_to_end();
                            break;
                        }

                        let _idx = black_box(idx);
                        let _value = black_box(value);
                    }
                }
                _ => {
                    let mut puller = iter.chunk_puller(batch);
                    while let Some(chunk) = puller.pull() {
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
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn vec_string_skip_to_end(num_threads: usize, batch: usize) {
    for len in LEN {
        let vec: Vec<_> = (0..len).map(|i| i.to_string()).collect();
        run(&vec.into_con_iter(), num_threads, batch);
    }
}

#[test_matrix(
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn vec_usize_skip_to_end(num_threads: usize, batch: usize) {
    for len in LEN {
        let vec: Vec<_> = (0..len).map(|i| i as usize).collect();
        run(&vec.into_con_iter(), num_threads, batch);
    }
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
