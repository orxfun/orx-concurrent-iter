use super::{jagged_index::JaggedIndex, raw_slice::RawSlice, raw_vec::RawVec};

/// A slice of a jagged array which might be empty, a slice of a single vector,
/// or a series of slices of subsequent arrays of the jagged array.
pub struct RawJaggedSlice<'a, T> {
    vectors: &'a [RawVec<T>],
    begin: JaggedIndex,
    end: JaggedIndex,
    len: usize,
    num_slices: usize,
}

impl<T> Default for RawJaggedSlice<'_, T> {
    fn default() -> Self {
        Self {
            vectors: Default::default(),
            begin: Default::default(),
            end: Default::default(),
            len: 0,
            num_slices: 0,
        }
    }
}

impl<'a, T> RawJaggedSlice<'a, T> {
    pub(super) fn new(
        arrays: &'a [RawVec<T>],
        begin: JaggedIndex,
        end: JaggedIndex,
        len: usize,
    ) -> Self {
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
        }
    }

    /// Returns total number of elements within the jagged arrays slice.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the number of slices within this jagged array slices.
    pub fn num_slices(&self) -> usize {
        self.num_slices
    }

    /// Returns the `s`-th raw slice among the slices of this jagged array slice.
    ///
    /// Returns `None` if `s >= self.num_slices`.
    pub fn get_raw_slice(&self, s: usize) -> Option<RawSlice<T>> {
        match s < self.num_slices {
            true => {
                let f = self.begin.f + s;
                let vec = &self.vectors[f];

                let start = match s == 0 {
                    true => self.begin.i,
                    false => 0,
                };

                let end_exc = match s == self.num_slices - 1 {
                    false => vec.len(),
                    true => match self.end.i {
                        0 => {
                            core::panic!("todo: why?");
                            vec.len()
                        }
                        end => end,
                    },
                };

                Some(vec.raw_slice(start, end_exc))
            }
            false => None,
        }
    }

    /// Returns the `f`-th slice of the jagged slice.
    ///
    /// Returns None if `f` is out of bounds, or the corresponding slice is empty.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub fn get_slice(&self, s: usize) -> Option<&'a [T]> {
        match s < self.num_slices {
            true => {
                let f = self.begin.f + s;
                let slice = &self.vectors[f];

                let start = match s == 0 {
                    true => self.begin.i,
                    false => 0,
                };

                let end_exc = match s == self.num_slices - 1 {
                    false => slice.len(),
                    true => match self.end.i {
                        0 => slice.len(),
                        end => end,
                    },
                };

                let len = end_exc - start;

                slice.slice(start, len)
            }
            false => None,
        }
    }
}
