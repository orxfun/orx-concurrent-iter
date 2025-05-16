use crate::implementations::{
    jagged_arrays::{
        as_slice::{AsOwningSlice, AsSlice},
        index::JaggedIndex,
        indexer::JaggedIndexer,
        jagged_slice::RawJaggedSlice,
    },
    ptr_utils::take,
};
use std::{cmp::Ordering, marker::PhantomData};

/// Raw representation of a jagged array.
/// Internally, the jagged array is stored as a vector of `S` that can
/// be converted into a slice.
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
///
/// Once dropped, the owned raw jagged array will drop the elements and
/// allocation of its raw vectors.
pub struct RawJagged<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
    arrays: Vec<S>,
    len: usize,
    indexer: X,
    num_taken: Option<usize>,
    phantom: PhantomData<T>,
}

impl<S, T, X> RawJagged<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
    /// Creates the raw jagged array for the given `arrays` and `indexer`.
    ///
    /// If the total number of elements in all `arrays` is known, it can be passed in as `total_len`,
    /// which will be assumed to be correct.
    /// If `None` is passed as the total length, it will be computed as sum of all vectors.
    ///
    /// Once the jagged array is dropped, the elements and allocation of the vectors
    /// will also be dropped.
    pub fn new(arrays: Vec<S>, indexer: X, total_len: Option<usize>) -> Self {
        let len = total_len.unwrap_or_else(|| arrays.iter().map(|v| v.length()).sum());
        Self {
            arrays,
            len,
            indexer,
            num_taken: Some(0),
            phantom: PhantomData,
        }
    }

    /// Leaves this jagged array empty without anything to drop.
    ///
    /// The remaining elements with respect to `num_taken` together with the allocation, and hence,
    /// the responsibility to drop, are transferred to the returned raw jagged array.
    pub(super) fn into_remaining_iter(&mut self, num_taken: usize) -> Self {
        let jagged_to_drop = Self {
            arrays: unsafe {
                Vec::from_raw_parts(
                    self.arrays.as_mut_ptr(),
                    self.arrays.len(),
                    self.arrays.capacity(),
                )
            },
            len: self.len,
            indexer: self.indexer.clone(),
            num_taken: Some(num_taken),
            phantom: PhantomData,
        };

        self.arrays = Vec::new();
        self.len = 0;
        self.num_taken = None;

        jagged_to_drop
    }

    /// Creates an empty raw jagged array with the given `indexer`.
    pub fn empty(indexer: X) -> Self {
        Self::new(Default::default(), indexer, Some(0))
    }

    /// Total number of elements in the jagged array (`O(1)`).
    pub fn len(&self) -> usize {
        self.len
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
                    let i = self.arrays[f].length();
                    Some(JaggedIndex::new(f, i))
                }
            },
            Ordering::Greater => None,
        }
    }

    /// Jagged array when created with `new_as_owned` is responsible for dropping its elements and clear the
    /// allocation of the vectors. However, it also allows that some of the elements are taken out before the
    /// jagged array is dropped, see the [`take`] method. This is tracked by `num_taken` which is `Some(0)`
    /// on construction.
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
    /// jagged array is dropped. This is tracked by `num_taken` which is `Some(0)` on construction.
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

    /// Returns a reference to the `f`-th slice, returns None iF `f >= self.num_arrays()`.
    pub fn get(&self, f: usize) -> Option<&S> {
        self.arrays.get(f)
    }

    /// Returns a reference to the `f`-th slice, without bounds checking.
    ///
    /// # SAFETY
    ///
    /// `f` must be within bounds; i.e., `f < self.num_arrays()`.
    pub unsafe fn get_unchecked(&self, f: usize) -> &S {
        debug_assert!(f < self.arrays.len());
        unsafe { self.arrays.get_unchecked(f) }
    }

    /// Returns the raw jagged array slice containing all elements having positions in range `flat_begin..flat_end`
    /// of the flattened jagged array.
    ///
    /// Returns an empty slice if any of the indices are out of bounds or if `flat_end <= flat_begin`.
    pub fn slice(&self, flat_begin: usize, flat_end: usize) -> RawJaggedSlice<S, T> {
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
    pub fn slice_from(&self, flat_begin: usize) -> RawJaggedSlice<S, T> {
        self.slice(flat_begin, self.len)
    }
}

impl<S, T, X> Drop for RawJagged<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
    fn drop(&mut self) {
        if let Some(num_taken) = self.num_taken {
            // drop elements
            if let Some(begin) = self.jagged_index(num_taken) {
                let s = &self.arrays[begin.f];
                for p in begin.i..s.length() {
                    unsafe { s.drop_in_place(p) };
                }

                for f in (begin.f + 1)..self.arrays.len() {
                    let s = &self.arrays[f];
                    for p in 0..s.length() {
                        unsafe { s.drop_in_place(p) };
                    }
                }
            }

            // drop allocation
            for vec in &self.arrays {
                unsafe { vec.drop_allocation() };
            }
        }
    }
}
