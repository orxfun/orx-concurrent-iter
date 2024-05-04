use orx_concurrent_iter::*;
use test_case::test_matrix;

const NUM_RERUNS: usize = 1;

fn concurrent_iter<I: Iterator<Item = i64>>(
    num_threads: usize,
    batch: usize,
    iter: I,
    expected_sum: i64,
) {
    let iter = &iter.into_con_iter();

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
                    while let Some(chunk) = iter.next_chunk(batch) {
                        sum += chunk.values.sum::<i64>();
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
    [1, 8, 64, 256],
    [4, 16],
    [1, 4, 64],
    [false, true]
)]
fn consume_iter(len: usize, num_threads: usize, batch: usize, lag: bool) {
    for _ in 0..NUM_RERUNS {
        let source: Vec<_> = (0..len).map(|x| x as i64).collect();

        let expected_sum: i64 = source.iter().filter(|x| filter(**x, false)).map(map).sum();

        concurrent_iter(
            num_threads,
            batch,
            source.iter().filter(|x| filter(**x, lag)).map(map),
            expected_sum,
        )
    }
}

fn filter(x: i64, lag: bool) -> bool {
    if lag {
        std::thread::sleep(std::time::Duration::from_nanos(10));
    }
    x % 3 == 1
}

fn map(x: &i64) -> i64 {
    x * 2 - 10
}
