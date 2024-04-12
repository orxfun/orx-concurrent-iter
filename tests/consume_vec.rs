use orx_concurrent_iter::*;
use test_case::test_matrix;

const NUM_RERUNS: usize = 1;

fn concurrent_iter(num_threads: usize, batch: usize, vec: Vec<i64>) {
    let expected_sum: i64 = vec.iter().sum();
    let iter = &vec.into_con_iter();

    let sum: i64 = std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut sum = 0i64;
                if batch == 1 {
                    while let Some(next) = iter.next_id_and_value() {
                        sum += next.value;
                    }
                } else {
                    let mut more = true;
                    while more {
                        more = false;
                        let next = iter.next_chunk(batch);
                        for value in next.values() {
                            sum += value;
                            more = true;
                        }
                    }
                }
                sum
            }));
        }

        handles.into_iter().map(|x| x.join().expect("-")).sum()
    });

    assert_eq!(sum, expected_sum);
}

#[test_matrix(
    [1, 2, 4, 8, 64, 1024, 64*1024],
    [4 ,8, 16],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn consume_vec(len: usize, num_threads: usize, batch: usize) {
    for _ in 0..NUM_RERUNS {
        let source: Vec<_> = (0..len).map(|x| x as i64).collect();
        concurrent_iter(num_threads, batch, source);
    }
}

fn concurrent_iter_heap(num_threads: usize, batch: usize, vec: Vec<String>) {
    let iter = &vec.into_con_iter();

    let some_str: String = std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(move || {
                let mut str = String::new();
                if batch == 1 {
                    while let Some(next) = iter.next_id_and_value() {
                        str = next.value;
                    }
                } else {
                    let mut more = true;
                    while more {
                        more = false;
                        let next = iter.next_chunk(batch);
                        for value in next.values() {
                            str = value;
                            more = true;
                        }
                    }
                }
                str
            }));
        }

        handles
            .into_iter()
            .map(|x| x.join().expect("-"))
            .filter(|x| !x.is_empty())
            .last()
            .unwrap()
    });

    assert!(!some_str.is_empty());
}

#[test_matrix(
    [1, 2, 4, 8, 64, 1024, 4*1024],
    [4 ,8, 16],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn consume_vec_heap(len: usize, num_threads: usize, batch: usize) {
    for _ in 0..NUM_RERUNS {
        let source: Vec<_> = (0..len).map(|x| x.to_string()).collect();
        concurrent_iter_heap(num_threads, batch, source);
    }
}
