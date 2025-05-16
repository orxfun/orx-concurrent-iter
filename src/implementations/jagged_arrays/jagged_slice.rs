use super::{as_slice::AsOwningSlice, index::JaggedIndex};
use std::marker::PhantomData;

/// A slice of a jagged array which might be empty, a slice of a single vector,
/// or a series of slices of subsequent arrays of the jagged array.
pub struct RawJaggedSlice<'a, S, T>
where
    S: AsOwningSlice<T>,
{
    vectors: &'a [S],
    begin: JaggedIndex,
    end: JaggedIndex,
    len: usize,
    num_slices: usize,
    phantom: PhantomData<T>,
}

impl<S, T> Default for RawJaggedSlice<'_, S, T>
where
    S: AsOwningSlice<T>,
{
    fn default() -> Self {
        Self {
            vectors: Default::default(),
            begin: Default::default(),
            end: Default::default(),
            len: 0,
            num_slices: 0,
            phantom: PhantomData,
        }
    }
}

impl<'a, S, T> RawJaggedSlice<'a, S, T>
where
    S: AsOwningSlice<T>,
{
    /// Constructs a non-empty raw jagged slice.
    pub(super) fn new(arrays: &'a [S], begin: JaggedIndex, end: JaggedIndex, len: usize) -> Self {
        debug_assert!(begin.is_in_exc_bounds_of(&arrays));
        debug_assert!(end.is_in_exc_bounds_of(&arrays));
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
