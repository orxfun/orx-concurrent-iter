use crate::implementations::{
    jagged::JaggedIndexer,
    jagged_arrays::{AsSlice, raw_slice::RawSlice},
};

/// Raw representation of a reference to a jagged array.
/// Internally, the jagged array is stored as a vector of `RawSlice<T>`
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
pub struct RawJaggedRef<'a, T, X>
where
    X: JaggedIndexer,
{
    arrays: Vec<RawSlice<'a, T>>,
    len: usize,
    indexer: X,
}

impl<'a, T, X> RawJaggedRef<'a, T, X>
where
    X: JaggedIndexer,
{
    /// Creates the raw jagged array reference for the given `arrays` and `indexer`.
    pub fn new(arrays: Vec<RawSlice<'a, T>>, indexer: X, total_len: Option<usize>) -> Self {
        let len = total_len.unwrap_or_else(|| arrays.iter().map(|v| v.length()).sum());
        Self {
            arrays,
            len,
            indexer,
        }
    }
}
