use orx_concurrent_iter::*;
use test_case::test_matrix;

const NUM_RERUNS: usize = 1;

#[cfg(not(miri))]
const LEN: [usize; 6] = [1, 4, 1024, 2141, 2 * 1024, 4 * 1024 + 1];
#[cfg(miri)]
const LEN: [usize; 6] = [1, 4, 32, 33, 2 * 64, 4 * 64 + 1];

fn concurrent_iter_stack(num_threads: usize, batch: usize, vec: Vec<i64>, consume_till_end: bool) {
    let vec_len = vec.len();
    let expected_sum: Option<i64> = match consume_till_end {
        true => Some(vec.iter().sum()),
        false => None,
    };
    let iter = &vec.into_con_iter();

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
fn consume_vec_stack(num_threads: usize, batch: usize) {
    for consume_till_end in [true, false] {
        for _ in 0..NUM_RERUNS {
            for len in LEN {
                let source: Vec<_> = (0..len).map(|x| x as i64).collect();
                concurrent_iter_stack(num_threads, batch, source, consume_till_end);
            }
        }
    }
}

fn concurrent_iter_heap(
    num_threads: usize,
    batch: usize,
    vec: Vec<String>,
    consume_till_end: bool,
) {
    let vec_len = vec.len();
    let iter = &vec.into_con_iter();

    let some_str: String = std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut str = String::new();
                match consume_till_end {
                    true => match batch {
                        1 => {
                            while let Some(value) = iter.next() {
                                str = value;
                            }
                        }
                        _ => {
                            let mut puller = iter.chunk_puller(batch);
                            while let Some(chunk) = puller.pull() {
                                for value in chunk {
                                    str = value;
                                }
                            }
                        }
                    },
                    false => match batch {
                        1 => {
                            let until = vec_len - 1;
                            for _ in 0..until {
                                if let Some(value) = iter.next() {
                                    str = value;
                                }
                            }
                        }
                        _ => {
                            let until = vec_len.saturating_sub(batch);
                            let mut puller = iter.chunk_puller(batch);
                            while iter.try_get_len().expect("exists") < until {
                                if let Some(chunk) = puller.pull() {
                                    for value in chunk {
                                        str = value;
                                    }
                                }
                            }
                        }
                    },
                }
                str
            }));
        }

        let default_str = match consume_till_end {
            true => String::default(),
            false => "might-be-empty".to_string(),
        };

        handles
            .into_iter()
            .map(|x| x.join().expect("-"))
            .filter(|x| !x.is_empty())
            .last()
            .unwrap_or(default_str)
    });

    assert!(!some_str.is_empty());
}

#[test_matrix(
    [4, 8, 16],
    [1, 4, 64, 1024]
)]
fn consume_vec_heap(num_threads: usize, batch: usize) {
    for consume_till_end in [true, false] {
        for _ in 0..NUM_RERUNS {
            for len in LEN {
                let source: Vec<_> = (0..len).map(|x| x.to_string()).collect();
                concurrent_iter_heap(num_threads, batch, source, consume_till_end);
            }
        }
    }
}
