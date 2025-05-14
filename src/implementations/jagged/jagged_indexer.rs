use super::{jagged_index::JaggedIndex, raw_vec::RawVec};

pub trait JaggedIndexer: Clone {
    fn jagged_index<T>(&self, arrays: &[RawVec<T>], flat_index: usize) -> Option<JaggedIndex>;
}
