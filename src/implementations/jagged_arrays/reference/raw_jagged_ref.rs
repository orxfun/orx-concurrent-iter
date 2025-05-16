use crate::implementations::{jagged::JaggedIndexer, jagged_arrays::raw_slice::RawSlice};

/// Raw representation of a reference to a jagged array.
/// Internally, the jagged array is stored as a vector of `RawSlice<T>`
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
///
/// Once dropped, the owned raw jagged array will drop the elements and
/// allocation of its raw vectors.
pub struct RawJaggedRef<'a, T, X>
where
    X: JaggedIndexer,
{
    arrays: Vec<RawSlice<'a, T>>,
    len: usize,
    indexer: X,
}
