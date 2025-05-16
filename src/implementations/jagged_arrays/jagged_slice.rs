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

    /// Returns the `s`-th raw slice among the slices of this jagged array slice.
    ///
    /// Returns None if `f` is out of bounds, or the corresponding slice is empty.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub fn get_raw_slice(&self, s: usize) -> Option<&[T]> {
        match s < self.num_slices {
            true => {
                let f = self.begin.f + s;
                let vec = &self.vectors[f];

                let start = match s == 0 {
                    true => self.begin.i,
                    false => 0,
                };

                let end_exc = match s == self.num_slices - 1 {
                    false => vec.length(),
                    true => match self.end.i {
                        0 => vec.length(),
                        end => end,
                    },
                };

                let len = end_exc - start;
                debug_assert!(len > 0);

                Some(vec.slice(start, len))
            }
            false => None,
        }
    }
}
