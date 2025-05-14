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

    pub fn slice_len<T>(
        &self,
        arrays: &[RawVec<T>],
        begin: &JaggedIndex,
        end: &JaggedIndex,
    ) -> usize {
        let [begin, end] = [begin, end].map(|x| self.flat_index(arrays, x));
        match (begin, end) {
            (Some(begin), Some(end)) => end.saturating_sub(begin),
            _ => 0,
        }
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

    fn flat_index<T>(&self, arrays: &[RawVec<T>], jagged_index: &JaggedIndex) -> Option<usize> {
        let flat_index = unsafe { self.flat_index_unchecked(arrays, jagged_index) };
        (flat_index <= self.len).then_some(flat_index)
    }

    unsafe fn flat_index_unchecked<T>(
        &self,
        _arrays: &[RawVec<T>],
        jagged_index: &JaggedIndex,
    ) -> usize {
        jagged_index.f * self.n + jagged_index.i
    }
}

#[derive(Clone)]
pub struct GeneralJaggedIndexer {
    len: usize,
}

impl GeneralJaggedIndexer {
    pub fn new(len: usize) -> Self {
        Self { len }
    }
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

    fn flat_index<T>(&self, arrays: &[RawVec<T>], jagged_index: &JaggedIndex) -> Option<usize> {
        let flat_index = unsafe { self.flat_index_unchecked(arrays, jagged_index) };
        (flat_index <= self.len).then_some(flat_index)
    }

    unsafe fn flat_index_unchecked<T>(
        &self,
        arrays: &[RawVec<T>],
        jagged_index: &JaggedIndex,
    ) -> usize {
        let [f, i] = [jagged_index.f, jagged_index.i];
        let until: usize = arrays.iter().take(f).map(|x| x.len()).sum();
        until + i
    }
}
