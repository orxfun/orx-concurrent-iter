use std::cmp::Ordering;

use crate::implementations::jagged_arrays::{
    AsSlice, JaggedIndex, JaggedIndexer, jagged_slice::RawJaggedSlice, raw_slice::RawSlice,
};

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

    /// Total number of elements in the jagged array (`O(1)`).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the [`JaggedIndex`] of the element at the given `flat_index` position of the flattened
    /// jagged array.
    ///
    /// It returns `None` when `flat_index > self.len()`.
    /// Importantly note that it returns `Some` when `flat_index == self.len()` which is the exclusive bound
    /// of the collection.
    ///
    /// Returns:
    ///
    /// * `Some([f, i])` if `flat_index < self.len()` such that the element is located at the `f`-th array's
    ///   `i`-th position.
    /// * `Some([f*, i*])` if `flat_index = self.len()` such that `f* = self.len() - 1` and `i* = self.arrays()[f*].len()`.
    ///   In other words, this is the exclusive end of the jagged index range of the jagged array.
    /// * `None` if `flat_index > self.len()`.
    pub fn jagged_index(&self, flat_index: usize) -> Option<JaggedIndex> {
        match flat_index.cmp(&self.len) {
            Ordering::Less => Some(unsafe {
                // SAFETY: flat_index is within bounds
                self.indexer
                    .jagged_index_unchecked(&self.arrays, flat_index)
            }),
            Ordering::Equal => match self.arrays.is_empty() {
                true => None,
                false => {
                    let f = self.arrays.len() - 1;
                    let i = self.arrays[f].length();
                    Some(JaggedIndex::new(f, i))
                }
            },
            Ordering::Greater => None,
        }
    }

    /// Returns the raw jagged array slice containing all elements having positions in range `flat_begin..flat_end`
    /// of the flattened jagged array.
    ///
    /// Returns an empty slice if any of the indices are out of bounds or if `flat_end <= flat_begin`.
    pub fn slice(&self, flat_begin: usize, flat_end: usize) -> RawJaggedSlice<RawSlice<'a, T>, T> {
        match flat_end.saturating_sub(flat_begin) {
            0 => Default::default(),
            len => {
                let [begin, end] = [flat_begin, flat_end].map(|i| self.jagged_index(i));
                match (begin, end) {
                    (Some(begin), Some(end)) => RawJaggedSlice::new(&self.arrays, begin, end, len),
                    _ => Default::default(),
                }
            }
        }
    }

    /// Returns the raw jagged array slice for the flattened positions within range `flat_begin..self.len()`.
    pub fn slice_from(&self, flat_begin: usize) -> RawJaggedSlice<RawSlice<'a, T>, T> {
        self.slice(flat_begin, self.len)
    }
}
