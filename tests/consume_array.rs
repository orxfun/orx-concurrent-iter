use orx_concurrent_iter::*;
use test_case::test_matrix;

const NUM_RERUNS: usize = 1;

fn concurrent_iter(num_threads: usize, batch: usize, array: [i64; 1024]) {
    let expected_sum: i64 = array.iter().sum();
    let iter = &array.into_con_iter();

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
    [4 ,8, 16],
    [1, 2, 4, 5, 8, 64, 71, 1024, 1025]
)]
fn consume_array(num_threads: usize, batch: usize) {
    for _ in 0..NUM_RERUNS {
        let mut array = [0i64; 1024];
        for (i, x) in array.iter_mut().enumerate() {
            *x = i as i64;
        }
        concurrent_iter(num_threads, batch, array);
    }
}
