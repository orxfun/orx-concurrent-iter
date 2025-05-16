use crate::implementations::jagged_arrays::{AsSlice, JaggedIndexer, raw_slice::RawSlice};

/// Raw representation of a reference to a jagged array.
/// Internally, the jagged array is stored as a vector of `RawSlice<T>`
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
pub struct RawJaggedRef<'a, T, X>
where
    X:,
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
    ///
    /// If the total number of elements in all `arrays` is known, it can be passed in as `total_len`,
    /// which will be assumed to be correct.
    /// If `None` is passed as the total length, it will be computed as sum of all vectors.
    pub fn new(arrays: Vec<RawSlice<'a, T>>, indexer: X, total_len: Option<usize>) -> Self {
        let len = total_len.unwrap_or_else(|| arrays.iter().map(|v| v.length()).sum());
        Self {
            arrays,
            len,
            indexer,
        }
    }
}
