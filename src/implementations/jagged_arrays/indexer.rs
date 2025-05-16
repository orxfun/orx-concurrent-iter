use super::{Slices, as_raw_slice::AsRawSlice, index::JaggedIndex};
use orx_pseudo_default::PseudoDefault;

/// An indexer for the raw jagged arrays.
pub trait JaggedIndexer: Clone + PseudoDefault + Send + Sync {
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
    unsafe fn jagged_index_unchecked<'a, T: 'a>(
        &self,
        arrays: &impl Slices<'a, T>,
        flat_index: usize,
    ) -> JaggedIndex;

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
    unsafe fn jagged_index_unchecked_from_slice<'a, T: 'a>(
        &self,
        arrays: &[impl AsRawSlice<T>],
        flat_index: usize,
    ) -> JaggedIndex;
}
