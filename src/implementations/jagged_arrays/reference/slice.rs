use super::raw_jagged_ref::RawJaggedRef;
use crate::implementations::jagged_arrays::{JaggedIndex, JaggedIndexer, as_slice::AsSlice};

/// A slice of a jagged array which might be empty, a slice of a single vector,
/// or a series of slices of subsequent arrays of the jagged array.
pub struct RawJaggedSlice<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    jagged: RawJaggedRef<'a, T, S, X>,
    begin: JaggedIndex,
    end: JaggedIndex,
    len: usize,
    num_slices: usize,
}

impl<'a, T, S, X> Default for RawJaggedSlice<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    fn default() -> Self {
        Self {
            jagged: Default::default(),
            begin: Default::default(),
            end: Default::default(),
            len: Default::default(),
            num_slices: Default::default(),
        }
    }
}

impl<'a, T, S, X> RawJaggedSlice<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    pub(super) fn new(
        jagged: RawJaggedRef<'a, T, S, X>,
        begin: JaggedIndex,
        end: JaggedIndex,
        len: usize,
    ) -> Self {
        // TODO: debug assert bounds
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
            jagged,
            begin,
            end,
            len,
            num_slices,
        }
    }

    /// Returns total number of elements within the jagged arrays slice (`O(1)`).
    pub fn len(&self) -> usize {
        self.len
    }

    pub(super) fn get_slice(&self, s: usize) -> Option<&'a [T]> {
        let f = self.begin.f + s;
        match self.jagged.len_of(f) {
            Some(arr_len) => {
                let begin_i = match s == 0 {
                    true => self.begin.i,
                    false => 0,
                };

                let end_exc = match s == self.num_slices - 1 {
                    false => arr_len,
                    true => match self.end.i {
                        0 => arr_len,
                        end => end,
                    },
                };

                let len = end_exc - begin_i;
                debug_assert!(len > 0);

                self.jagged.slice(f, begin_i, len)
            }
            None => None,
        }
    }
}
