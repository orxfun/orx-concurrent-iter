use super::{
    jagged_index::JaggedIndex, jagged_indexer::JaggedIndexer, raw_slice::RawSlice, raw_vec::RawVec,
};
use crate::implementations::ptr_utils::take;
use std::cmp::Ordering;

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
                    let i = self.arrays[f].len();
                    Some(JaggedIndex::new(f, i))
                }
            },
            Ordering::Greater => None,
        }
    }

    /// Jagged array when created with `new_as_owned` is responsible for dropping its elements and clear the
    /// allocation of the vectors. However, it also allows that some of the elements are taken out before the
    /// jagged array is dropped, see the [`take`] method. This is tracked by `num_taken` which is
    /// * `Some(0)` on construction using `new_as_owned`,
    /// * `None` on construction using `new_as_reference`.
    ///
    /// This method overwrites `num_taken`:
    ///
    /// * when `Some(n)`, the first `n` elements of the jagged array will not be dropped when
    ///   this `RawJagged` is dropped. The remaining elements and the allocations will be dropped.
    /// * when `None`, neither any of the elements nor the allocations will be dropped.
    ///
    /// [`take`]: Self::take
    ///
    /// # Safety
    ///
    /// This method is unsafe due to the following use cases which would lead to undefined behavior.
    ///
    /// If the jagged array is created as a reference:
    /// * we intend the drop the vectors outside of this jagged array,
    /// * therefore, calling this method with `Some(n)` would lead to attempting the elements twice.
    ///
    /// If the jagged array is created as owned:
    /// * we intend to drop the vectors when we drop this jagged array,
    /// * in this case, calling this method with `None` would cause the memory to leak.
    ///
    /// If the jagged array is crated as owned, and if we call this method with `Some(n)`:
    /// * we must make sure that all elements with flat indices within range `0..n` must be taken manually
    ///   by the [`take`] method; and hence, will be dropped externally;
    /// * if an element within `0..n` is not taken, the corresponding element will leak,
    /// * if an element outside `0..n` is taken, there will be two attempts to drop the same element.
    ///
    /// Note that it is not required to call `set_num_taken` immediately after calling `take`.
    /// The following sequence of events is perfectly safe:
    ///
    /// * `jagged.take(2)`
    /// * `jagged.take(0)`
    /// * `jagged.take(1)`
    /// * `jagged.set_num_taken(3)`
    /// * `drop(jagged)`
    pub unsafe fn set_num_taken(&mut self, num_taken: Option<usize>) {
        self.num_taken = num_taken;
    }

    /// Jagged array when created with `new_as_owned` is responsible for dropping its elements and clear the
    /// allocation of the vectors. However, it also allows that some of the elements are taken out before the
    /// jagged array is dropped. This is tracked by `num_taken` which is
    /// * `Some(0)` on construction using `new_as_owned`,
    /// * `None` on construction using `new_as_reference`.
    ///
    /// See [`set_num_taken`] to update the number of manually taken elements in order to avoid dropping the
    /// same element twice.
    ///
    /// This method returns currently value of `num_taken`.
    ///
    /// [`set_num_taken`]: Self::set_num_taken
    pub fn num_taken(&self) -> Option<usize> {
        self.num_taken
    }

    /// Takes the element at the `flat-index`-th position of the flattened jagged array.
    ///
    /// # Safety
    ///
    /// This method safely takes out the element; however, leaves the `flat-index`-th position uninitialized.
    ///
    /// Therefore, jagged array must not attempt to drop the element at this position.
    /// This can be controlled using the [`set_num_taken`] method.
    ///
    /// [`set_num_taken`]: Self::set_num_taken
    pub unsafe fn take(&self, flat_index: usize) -> Option<T> {
        self.jagged_index(flat_index).map(|idx| {
            let vec = &self.arrays[idx.f];
            let ptr = unsafe { vec.ptr_at(idx.i) as *mut T }; // index is in bounds
            unsafe { take(ptr) }
        })
    }

    /// Returns a reference to the element at the `flat_index`-th position of the flattened jagged array.
    pub fn get(&self, flat_index: usize) -> Option<&T> {
        self.jagged_index(flat_index).map(|idx| {
            // SAFETY: jagged_index call ensures that idx.i is in bounds
            unsafe { self.arrays[idx.f].get_unchecked(idx.i) }
        })
    }

    /// Returns the `f`-th array of the jagged array as a raw slice.
    ///
    /// Returns `None` if `f >= self.num_arrays()`.
    pub fn get_raw_slice(&self, f: usize) -> Option<RawSlice<T>> {
        self.arrays.get(f).map(|vec| vec.as_raw_slice())
    }
}

impl<T, X> Drop for RawJagged<T, X>
where
    X: JaggedIndexer,
{
    fn drop(&mut self) {
        if let Some(num_taken) = self.num_taken {
            // drop elements
            if let Some(begin) = self.jagged_index(num_taken) {
                let [f, i] = [begin.f, begin.i];
                unsafe { self.arrays[f].drop_elements_in_place(i) };

                for f in (f + 1)..self.arrays.len() {
                    unsafe { self.arrays[f].drop_elements_in_place(0) };
                }
            }

            // drop allocation
            for vec in &self.arrays {
                unsafe { vec.drop_allocation() };
            }
        }
    }
}
