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
    vectors: Vec<RawVec<T>>,
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
            vectors: self.vectors.clone(),
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
    fn new(vectors: Vec<RawVec<T>>, indexer: X, droppable: bool) -> Self {
        let len = vectors.iter().map(|v| v.len()).sum();
        let num_taken = droppable.then_some(0);

        Self {
            vectors,
            len,
            indexer,
            num_taken,
        }
    }

    /// Creates the raw jagged array for the given `vectors` and `indexer`.
    ///
    /// Once the jagged array is dropped, the elements and allocation of the vectors
    /// will also be dropped.
    pub fn new_as_owned(vectors: Vec<RawVec<T>>, indexer: X) -> Self {
        Self::new(vectors, indexer, true)
    }

    /// Creates the raw jagged array for the given `vectors` and `indexer`.
    ///
    /// Dropping this jagged array will not drop the underlying elements or allocations.
    pub fn new_as_reference(vectors: Vec<RawVec<T>>, indexer: X) -> Self {
        Self::new(vectors, indexer, false)
    }
}
