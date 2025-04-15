use orx_concurrent_iter::*;
use test_case::test_matrix;

const N: usize = 1024;
const NUM_RERUNS: usize = 1;

fn concurrent_iter_stack(
    num_threads: usize,
    batch: usize,
    array: [i64; N],
    consume_till_end: bool,
) {
    let vec_len = array.len();
    let expected_sum: Option<i64> = match consume_till_end {
        true => Some(array.iter().sum()),
        false => None,
    };
    let iter = &array.into_con_iter();

    let sum: i64 = std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut sum = 0i64;
                match consume_till_end {
                    true => match batch {
                        1 => {
                            while let Some(value) = iter.next() {
                                sum += value;
                            }
                        }
                        _ => {
                            let mut puller = iter.chunk_puller(batch);
                            while let Some(chunk) = puller.pull() {
                                for value in chunk {
                                    sum += value;
                                }
                            }
                        }
                    },
                    false => match batch {
                        1 => {
                            let until = vec_len - 1;
                            for _ in 0..until {
                                if let Some(value) = iter.next() {
                                    sum += value;
                                }
                            }
                        }
                        _ => {
                            let until = vec_len.saturating_sub(batch);
                            let mut puller = iter.chunk_puller(batch);
                            while iter.try_get_len().expect("exists") < until {
                                if let Some(chunk) = puller.pull() {
                                    for value in chunk {
                                        sum += value;
                                    }
                                }
                            }
                        }
                    },
                }

                sum
            }));
        }

        handles.into_iter().map(|x| x.join().expect("-")).sum()
    });
    if let Some(expected_sum) = expected_sum {
        assert_eq!(sum, expected_sum);
    }
}

#[test_matrix(
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn consume_array_stack(num_threads: usize, batch: usize) {
    for consume_till_end in [true, false] {
        for _ in 0..NUM_RERUNS {
            let mut source = [0i64; 1024];
            for (i, x) in source.iter_mut().enumerate() {
                *x = i as i64;
            }
            concurrent_iter_stack(num_threads, batch, source, consume_till_end);
        }
    }
}
