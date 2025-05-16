use super::{as_slice::AsSlice, index::JaggedIndex};

/// An indexer for the raw jagged arrays.
pub trait JaggedIndexer: Clone + Send + Sync {
    /// Returns the jagged index of the element `flat_index`-th position if the raw jagged array
    /// defined by the `arrays` collection would have been flattened.
    ///
    /// The model expects `total_len` to be equal to `arrays.iter().map(|x| x.len()).sum()`.
    ///
    /// Returns `None` if `flat_index > total_len`.
    ///
    /// Importantly note that it returns Some when `flat_index` is equal to the total length of the
    /// jagged array, which represents the exclusive bound of the jagged indices.
    fn jagged_index<T>(
        &self,
        total_len: usize,
        arrays: &[impl AsSlice<T>],
        flat_index: usize,
    ) -> Option<JaggedIndex>;

    /// Returns the jagged index of the element `flat_index`-th position if the raw jagged array
    /// defined by the `arrays` collection would have been flattened.
    ///
    /// Unlike `jagged_index`, this method expects `flat_index <= arrays.iter().map(|x| x.len()).sum()`,
    /// and omits bounds checks.
    ///
    /// # SAFETY
    ///
    /// Calling this method with an index greater than the total length of the jagged array might
    /// possibly lead to undefined behavior.
    unsafe fn jagged_index_unchecked<T>(
        &self,
        arrays: &[impl AsSlice<T>],
        flat_index: usize,
    ) -> JaggedIndex;
}
