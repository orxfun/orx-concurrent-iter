use super::jagged_index::JaggedIndex;

pub trait JaggedIndexer {
    fn jagged_index(&self, flat_index: usize) -> Option<JaggedIndex>;
}
