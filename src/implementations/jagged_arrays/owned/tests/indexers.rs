use crate::implementations::jagged_arrays::{
    as_slice::AsSlice, index::JaggedIndex, indexer::JaggedIndexer,
};

#[derive(Clone)]
pub struct MatrixIndexer {
    n: usize,
}

impl MatrixIndexer {
    pub fn new(n: usize) -> Self {
        Self { n }
    }

    fn flat_index<S, T>(
        &self,
        total_len: usize,
        arrays: &[S],
        jagged_index: &JaggedIndex,
    ) -> Option<usize>
    where
        S: AsSlice<T>,
    {
        let flat_index = unsafe { self.flat_index_unchecked(arrays, jagged_index) };
        (flat_index <= total_len).then_some(flat_index)
    }

    unsafe fn flat_index_unchecked<S, T>(&self, _arrays: &[S], jagged_index: &JaggedIndex) -> usize
    where
        S: AsSlice<T>,
    {
        jagged_index.f * self.n + jagged_index.i
    }

    pub fn slice_len<S, T>(&self, arrays: &[S], begin: &JaggedIndex, end: &JaggedIndex) -> usize
    where
        S: AsSlice<T>,
    {
        let total_len = arrays.iter().map(|x| x.length()).sum();
        let [begin, end] = [begin, end].map(|x| self.flat_index(total_len, arrays, x));
        match (begin, end) {
            (Some(begin), Some(end)) => end.saturating_sub(begin),
            _ => 0,
        }
    }
}

impl JaggedIndexer for MatrixIndexer {
    fn jagged_index<T>(
        &self,
        total_len: usize,
        arrays: &[impl AsSlice<T>],
        flat_index: usize,
    ) -> Option<JaggedIndex> {
        match flat_index <= total_len {
            true => Some(unsafe { self.jagged_index_unchecked(arrays, flat_index) }),
            false => None,
        }
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        _arrays: &[impl AsSlice<T>],
        flat_index: usize,
    ) -> JaggedIndex {
        let f = flat_index / self.n;
        let i = flat_index % self.n;
        JaggedIndex::new(f, i)
    }
}

#[derive(Clone)]
pub struct GeneralJaggedIndexer;

impl JaggedIndexer for GeneralJaggedIndexer {
    fn jagged_index<T>(
        &self,
        total_len: usize,
        arrays: &[impl AsSlice<T>],
        flat_index: usize,
    ) -> Option<JaggedIndex> {
        match flat_index <= total_len {
            true => Some(unsafe { self.jagged_index_unchecked(arrays, flat_index) }),
            false => None,
        }
    }

    unsafe fn jagged_index_unchecked<T>(
        &self,
        arrays: &[impl AsSlice<T>],
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
