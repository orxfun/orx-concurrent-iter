use super::slice::RawJaggedSlice;
use crate::implementations::jagged_arrays::{
    AsRawSlice, Slices, as_slice::AsSlice, index::JaggedIndex, indexer::JaggedIndexer,
};
use core::{cmp::Ordering, marker::PhantomData};
use orx_pseudo_default::PseudoDefault;

/// Raw representation of a reference to a jagged array.
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
pub struct RawJaggedRef<'a, T, S, X>
where
    T: 'a,
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    arrays: S,
    len: usize,
    indexer: X,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, S, X> Default for RawJaggedRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    fn default() -> Self {
        Self {
            arrays: S::empty(),
            len: Default::default(),
            indexer: PseudoDefault::pseudo_default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, T, S, X> Clone for RawJaggedRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    fn clone(&self) -> Self {
        Self {
            arrays: self.arrays.clone(),
            len: self.len.clone(),
            indexer: self.indexer.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'a, T, S, X> RawJaggedRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    /// Creates a new raw jagged array of references.
    pub fn new(arrays: S, indexer: X, total_len: Option<usize>) -> Self {
        let len = total_len.unwrap_or_else(|| arrays.lengths().sum());
        Self {
            arrays,
            len,
            indexer,
            phantom: PhantomData,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn num_slices(&self) -> usize {
        self.arrays.num_slices()
    }

    pub(super) fn len_of(&self, f: usize) -> Option<usize> {
        self.arrays.slice_at(f).map(|x| x.as_slice().len())
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
    pub(super) fn jagged_index(&self, flat_index: usize) -> Option<JaggedIndex> {
        match flat_index.cmp(&self.len) {
            Ordering::Less => Some(unsafe {
                // SAFETY: flat_index is within bounds
                self.indexer
                    .jagged_index_unchecked(&self.arrays, flat_index)
            }),
            Ordering::Equal => match self.arrays.num_slices() == 0 {
                true => None,
                false => {
                    let f = self.arrays.num_slices() - 1;
                    // SAFETY: f is in bounds since arrays is not empty
                    let i = unsafe { self.arrays.slice_at_unchecked(f) }.length();
                    Some(JaggedIndex::new(f, i))
                }
            },
            Ordering::Greater => None,
        }
    }

    pub(super) fn slice(&self, f: usize, begin_within_slice: usize, len: usize) -> Option<&'a [T]> {
        self.arrays.slice_at(f).and_then(|array| {
            let array = array.as_slice();
            (begin_within_slice < array.len()).then(|| {
                let ptr = unsafe { array.as_ptr().add(begin_within_slice) };
                unsafe { core::slice::from_raw_parts(ptr, len) }
            })
        })
    }

    pub(super) fn jagged_slice(
        &self,
        flat_begin: usize,
        flat_end: usize,
    ) -> RawJaggedSlice<'a, T, S, X> {
        match flat_end.saturating_sub(flat_begin) {
            0 => Default::default(),
            len => {
                let [begin, end] = [flat_begin, flat_end].map(|i| self.jagged_index(i));
                match (begin, end) {
                    (Some(begin), Some(end)) => RawJaggedSlice::new(self.clone(), begin, end, len),
                    _ => Default::default(),
                }
            }
        }
    }

    pub(super) fn get(&self, flat_index: usize) -> Option<&'a T> {
        self.jagged_index(flat_index).map(|x| {
            // SAFETY: both f and i are in bounds
            unsafe { self.arrays.slice_at_unchecked(x.f).get_unchecked(x.i) }
        })
    }
}
