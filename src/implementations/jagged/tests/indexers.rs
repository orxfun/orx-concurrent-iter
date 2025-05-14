use crate::implementations::jagged::{
    jagged_index::JaggedIndex, jagged_indexer::JaggedIndexer, raw_vec::RawVec,
};

#[derive(Clone)]
pub struct MatrixIndexer {
    n: usize,
    len: usize,
}

impl MatrixIndexer {
    pub fn new(n: usize) -> Self {
        Self { n, len: n * n }
    }
}

impl JaggedIndexer for MatrixIndexer {
    fn jagged_index<T>(&self, arrays: &[RawVec<T>], flat_index: usize) -> Option<JaggedIndex> {
        match flat_index <= self.len {
            true => Some(unsafe { self.jagged_index_unchecked(arrays, flat_index) }),
            false => None,
        }
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        _arrays: &[RawVec<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        let f = flat_index / self.n;
        let i = flat_index % self.n;
        JaggedIndex::new(f, i)
    }
}

#[derive(Clone)]
pub struct GeneralJaggedIndexer {
    lengths: Vec<usize>,
    len: usize,
}

impl JaggedIndexer for GeneralJaggedIndexer {
    fn jagged_index<T>(&self, arrays: &[RawVec<T>], flat_index: usize) -> Option<JaggedIndex> {
        todo!()
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        arrays: &[RawVec<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        match flat_index == self.len {
            true => match self.lengths.last() {
                Some(last_len) => JaggedIndex::new(self.lengths.len() - 1, *last_len),
                None => JaggedIndex::new(0, 0),
            },
            false => {
                let mut idx = flat_index;
                let [mut f, mut i] = [0, 0];
                let mut current_f = 0;
                while idx > 0 {
                    let current_len = lengths[current_f];
                    match current_len > idx {
                        true => {
                            i = idx;
                            idx = 0;
                        }
                        false => {
                            f += 1;
                            idx -= current_len;
                        }
                    }
                    current_f += 1;
                }
                [f, i]
            }
        }
    }
}

// fn jagged_indexer() -> impl Fn(usize) -> [usize; 2] + Clone {
//     let lengths = vec![2, 3, 1, 4];
//     move |idx| match idx == lengths.iter().sum::<usize>() {
//         true => [lengths.len() - 1, lengths[lengths.len() - 1]],
//         false => {
//             let mut idx = idx;
//             let [mut f, mut i] = [0, 0];
//             let mut current_f = 0;
//             while idx > 0 {
//                 let current_len = lengths[current_f];
//                 match current_len > idx {
//                     true => {
//                         i = idx;
//                         idx = 0;
//                     }
//                     false => {
//                         f += 1;
//                         idx -= current_len;
//                     }
//                 }
//                 current_f += 1;
//             }
//             [f, i]
//         }
//     }
// }
