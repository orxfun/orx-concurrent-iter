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
    len: usize,
}

impl JaggedIndexer for GeneralJaggedIndexer {
    fn jagged_index<T>(&self, arrays: &[RawVec<T>], flat_index: usize) -> Option<JaggedIndex> {
        match flat_index <= self.len {
            true => Some(unsafe { self.jagged_index_unchecked(arrays, flat_index) }),
            false => None,
        }
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        arrays: &[RawVec<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        assert!(flat_index <= self.len);

        match flat_index == self.len {
            true => match arrays.iter().map(|x| x.len()).enumerate().last() {
                Some((f, last_len)) => JaggedIndex::new(f, last_len),
                None => JaggedIndex::new(0, 0),
            },
            false => {
                let mut idx = flat_index;
                let [mut f, mut i] = [0, 0];
                let mut current_f = 0;
                while idx > 0 {
                    let current_len = arrays[current_f].len();
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
                JaggedIndex::new(f, i)
            }
        }
    }
}
