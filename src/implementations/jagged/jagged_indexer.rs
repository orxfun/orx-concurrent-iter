use super::jagged_index::JaggedIndex;

pub trait JaggedIndexer: Clone {
    fn jagged_index(&self, flat_index: usize) -> Option<JaggedIndex>;
}
