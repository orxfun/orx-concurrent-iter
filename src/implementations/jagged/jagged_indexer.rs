use super::{jagged_index::JaggedIndex, raw_vec::RawVec};

/// An indexer for the raw jagged arrays.
pub trait JaggedIndexer: Clone {
    /// Returns the jagged index of the element `flat_index`-th position if the raw jagged array
    /// defined by the `arrays` collection would have been flattened.
    ///
    /// Returns `None` if `flat_index > arrays.iter().map(|x| x.len()).sum()`.
    ///
    /// Importantly note that it returns Some when `flat_index` is equal to the total length of the
    /// jagged array, which represents the exclusive bound of the jagged indices.
    fn jagged_index<T>(&self, arrays: &[RawVec<T>], flat_index: usize) -> Option<JaggedIndex>;

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
        arrays: &[RawVec<T>],
        flat_index: usize,
    ) -> JaggedIndex;

    /// Returns the flat index of the element at the `jagged_index`-th position of the raw jagged
    /// array defined by the `arrays`.
    ///
    /// Returns `None` if `jagged_index` is greater than the next index of the last element.
    ///
    /// Importantly note that it returns Some(total_len) when `jagged_index` represents the `total_len`-th
    /// element of the jagged array, which represents the exclusive bound.
    fn flat_index<T>(&self, arrays: &[RawVec<T>], jagged_index: &JaggedIndex) -> Option<usize>;

    /// Returns the flat index of the element at the `jagged_index`-th position of the raw jagged
    /// array defined by the `arrays`.
    ///
    /// Importantly note that it returns total_len when `jagged_index` represents the `total_len`-th
    /// element of the jagged array, which represents the exclusive bound.
    ///
    /// # SAFETY
    ///
    /// Calling this method with an index greater than the total length of the jagged array might
    /// possibly lead to undefined behavior.
    unsafe fn flat_index_unchecked<T>(
        &self,
        arrays: &[RawVec<T>],
        jagged_index: &JaggedIndex,
    ) -> usize;
}
