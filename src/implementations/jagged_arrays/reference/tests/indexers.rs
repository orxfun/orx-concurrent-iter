use crate::implementations::jagged_arrays::{
    as_raw_slice::AsRawSlice, index::JaggedIndex, indexer::JaggedIndexer,
};
use orx_pseudo_default::PseudoDefault;

#[derive(Clone)]
pub struct MatrixIndexer {
    n: usize,
}

impl MatrixIndexer {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl PseudoDefault for MatrixIndexer {
    fn pseudo_default() -> Self {
        Self::new(1)
    }
}

impl JaggedIndexer for MatrixIndexer {
    fn jagged_index<T>(
        &self,
        total_len: usize,
        arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> Option<JaggedIndex> {
        match flat_index <= total_len {
            true => Some(unsafe { self.jagged_index_unchecked(arrays, flat_index) }),
            false => None,
        }
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        _arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        let f = flat_index / self.n;
        let i = flat_index % self.n;
        JaggedIndex::new(f, i)
    }
}

#[derive(Clone)]
pub struct GeneralJaggedIndexer;

impl PseudoDefault for GeneralJaggedIndexer {
    fn pseudo_default() -> Self {
        Self
    }
}

impl JaggedIndexer for GeneralJaggedIndexer {
    fn jagged_index<T>(
        &self,
        total_len: usize,
        arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> Option<JaggedIndex> {
        match flat_index <= total_len {
            true => Some(unsafe { self.jagged_index_unchecked(arrays, flat_index) }),
            false => None,
        }
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        let mut idx = flat_index;
        let [mut f, mut i] = [0, 0];
        let mut current_f = 0;
        while idx > 0 {
            let current_len = arrays[current_f].length();
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
