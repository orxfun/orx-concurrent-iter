use super::{jagged_indexer::JaggedIndexer, raw_vec::RawVec};

/// Raw representation of a jagged array.
/// Internally, the jagged array is stored as a vector of raw vectors.
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
///
/// Depending on how it is constructed, might or might not drop
/// the elements and allocation of its raw vectors;
/// see [`new_as_owned`] and [`new_as_reference`] constructors.
///
/// [`new_as_owned`]: Self::new_as_owned
/// [`new_as_reference`]: Self::new_as_reference
pub struct RawJagged<T, X>
where
    X: JaggedIndexer,
{
    arrays: Vec<RawVec<T>>,
    len: usize,
    indexer: X,
    num_taken: Option<usize>,
}

impl<T, X> Clone for RawJagged<T, X>
where
    X: JaggedIndexer,
{
    fn clone(&self) -> Self {
        Self {
            arrays: self.arrays.clone(),
            len: self.len,
            indexer: self.indexer.clone(),
            num_taken: self.num_taken,
        }
    }
}

impl<T, X> RawJagged<T, X>
where
    X: JaggedIndexer,
{
    fn new(arrays: Vec<RawVec<T>>, indexer: X, droppable: bool, total_len: Option<usize>) -> Self {
        let len = total_len.unwrap_or_else(|| arrays.iter().map(|v| v.len()).sum());
        let num_taken = droppable.then_some(0);

        Self {
            arrays,
            len,
            indexer,
            num_taken,
        }
    }

    /// Creates the raw jagged array for the given `arrays` and `indexer`.
    ///
    /// If the total number of elements in all `arrays` is known, it can be passed in as `total_len`,
    /// which will be assumed to be correct.
    /// If `None` is passed as the total length, it will be computed as sum of all vectors.
    ///
    /// Once the jagged array is dropped, the elements and allocation of the vectors
    /// will also be dropped.
    pub fn new_as_owned(arrays: Vec<RawVec<T>>, indexer: X, total_len: Option<usize>) -> Self {
        Self::new(arrays, indexer, true, total_len)
    }

    /// Creates the raw jagged array for the given `arrays` and `indexer`.
    ///
    /// If the total number of elements in all `arrays` is known, it can be passed in as `total_len`,
    /// which will be assumed to be correct.
    /// If `None` is passed as the total length, it will be computed as sum of all vectors.
    ///
    /// Dropping this jagged array will not drop the underlying elements or allocations.
    pub fn new_as_reference(arrays: Vec<RawVec<T>>, indexer: X, total_len: Option<usize>) -> Self {
        Self::new(arrays, indexer, false, total_len)
    }

    /// Total number of elements in the jagged array (`O(1)`).
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a reference to raw vectors of the jagged array.
    pub fn arrays(&self) -> &[RawVec<T>] {
        &self.arrays
    }

    /// Returns number of arrays of the jagged array.
    pub fn num_arrays(&self) -> usize {
        self.arrays.len()
    }
}
