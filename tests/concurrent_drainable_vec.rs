use orx_concurrent_iter::*;
use std::ops::Range;
use test_case::test_matrix;

#[derive(Clone, Debug)]
struct VecAndRange(Vec<String>, Range<usize>);

impl VecAndRange {
    fn new(n: usize) -> VecAndRange {
        let vec: Vec<_> = (0..n).map(|x| x.to_string()).collect();
        let range = match n % 4 {
            0 => 0..n,
            1 => 0..(n / 2),
            2 => core::cmp::min(n.saturating_sub(1), 3)..n,
            _ => {
                let a = core::cmp::min(n.saturating_sub(1), 3);
                let b = core::cmp::min(a + 10, n);
                a..b
            }
        };
        Self(vec, range)
    }
}

#[test_matrix(
    [0, 1, 2, 3, 71, 72, 73, 74],
    [true, false],
    [true, false]
)]
fn concurrent_drainable_vec(n: usize, use_all: bool, as_seq: bool) {
    let vec_and_range = VecAndRange::new(n);
    let VecAndRange(mut vec, range) = vec_and_range;
    let m = match use_all {
        true => range.len(),
        false => range.len() / 2,
    };

    let mut vec2 = vec.clone();
    let mut drained = vec![];
    {
        let mut draining = vec2.drain(range.clone());
        for _ in 0..m {
            drained.push(draining.next().unwrap());
        }
    }
    let expected = (vec2, drained);

    let mut drained = vec![];
    {
        match as_seq {
            true => {
                let mut draining = vec.con_drain(range).into_seq_iter();
                for _ in 0..m {
                    drained.push(draining.next().unwrap());
                }
            }
            false => {
                let draining = vec.con_drain(range);
                for _ in 0..m {
                    drained.push(draining.next().unwrap());
                }
            }
        }
    }
    let result = (vec, drained);
    assert_eq!(result, expected);
}
