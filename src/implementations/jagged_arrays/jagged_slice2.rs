use super::{as_slice::AsSlice, index::JaggedIndex, raw_slice::RawSlice};
use orx_v::{D1, NVec};
use std::marker::PhantomData;

/// A slice of a jagged array which might be empty, a slice of a single vector,
/// or a series of slices of subsequent arrays of the jagged array.
pub struct RawJaggedSlice<'a, V, S, T>
where
    S: AsSlice<T> + 'a,
    V: NVec<D1, &'a [S]>,
{
    vectors: V,
    begin: JaggedIndex,
    end: JaggedIndex,
    len: usize,
    num_slices: usize,
    phantom: PhantomData<&'a (S, T)>,
}

impl<'a, V, S, T> RawJaggedSlice<'a, V, S, T>
where
    S: AsSlice<T> + 'a,
    V: NVec<D1, &'a [S]>,
{
    /// Constructs a non-empty raw jagged slice.
    pub(super) fn new(arrays: V, begin: JaggedIndex, end: JaggedIndex, len: usize) -> Self {
        // TODO: add back the assertions
        // debug_assert!(begin.is_in_exc_bounds_of(&arrays));
        // debug_assert!(end.is_in_exc_bounds_of(&arrays));
        debug_assert!(begin <= end);

        let num_slices = match begin.f == end.f {
            true => match begin.i < end.i {
                true => 1,
                false => 0,
            },
            false => {
                const FIRST: usize = 1;
                let last = match end.i > 0 {
                    true => 1,
                    false => 0,
                };
                let middle = end.f - begin.f - 1;
                FIRST + last + middle
            }
        };

        Self {
            vectors: arrays,
            begin,
            end,
            len,
            num_slices,
            phantom: PhantomData,
        }
    }

    /// Returns total number of elements within the jagged arrays slice (`O(1)`).
    pub fn len(&self) -> usize {
        self.len
    }
}
