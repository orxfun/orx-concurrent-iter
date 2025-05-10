use crate::implementations::jagged::{raw_jagged::RawJagged, raw_vec::RawVec};

#[cfg(miri)]
const N: usize = 11;
#[cfg(not(miri))]
const N: usize = 66;

fn get_matrix(n: usize) -> Vec<Vec<String>> {
    let mut matrix = Vec::new();
    for i in 0..n {
        matrix.push(
            ((i * n)..((i + 1) * n))
                .map(|x| (10 + x).to_string())
                .collect(),
        );
    }
    matrix
}

fn matrix_indexer(n: usize) -> impl Fn(usize) -> [usize; 2] + Clone {
    move |idx| {
        let f = idx / n;
        let i = idx % n;
        [f, i]
    }
}

#[test]
fn enumeration() {
    let n = 2;
    let matrix = get_matrix(n);
    let slices: Vec<_> = matrix.iter().map(|x| RawVec::from(x.as_slice())).collect();
    let jagged = RawJagged::new(slices, matrix_indexer(n), false);
    // let iter = ConIterJaggedOwned::new(jagged, 0);

    // assert_eq!(iter.next(), Some(10.to_string()));
    // assert_eq!(iter.next_with_idx(), Some((1, 11.to_string())));
    // assert_eq!(iter.next(), Some(12.to_string()));
    // assert_eq!(iter.next_with_idx(), Some((3, 13.to_string())));
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next_with_idx(), None);
    // assert_eq!(iter.next(), None);
    // assert_eq!(iter.next_with_idx(), None);
}
