use super::{
    jagged_index::JaggedIndex, raw_jagged_slice_iter_owned::RawJaggedSliceIterOwned,
    raw_jagged_slice_iter_ref::RawJaggedSliceIterRef, raw_slice::RawSlice, raw_vec::RawVec,
};

pub struct RawJaggedSlice<'a, T> {
    vectors: &'a [RawVec<T>],
    begin: JaggedIndex,
    end: JaggedIndex,
    len: usize,
    num_slices: usize,
}

impl<'a, T> Default for RawJaggedSlice<'a, T> {
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
    pub fn new(
        vectors: &'a [RawVec<T>],
        begin: JaggedIndex,
        end: JaggedIndex,
        known_len: Option<usize>,
    ) -> Self {
        assert!(begin.is_in_inc_bounds_of(&vectors));
        assert!(end.is_in_exc_bounds_of(&vectors));
        assert!(begin <= end);

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

        let len = match known_len {
            Some(x) => x,
            None => {
                let mut len = 0;
                for f in begin.f..=end.f {
                    if let Some(vec) = vectors.get(f) {
                        let slice_len = match f {
                            f if f == begin.f && f == end.f => end.i - begin.i,
                            f if f == begin.f => vec.len() - begin.i,
                            f if f == end.f => end.i,
                            _ => vec.len(),
                        };
                        len += slice_len
                    }
                }
                len
            }
        };

        Self {
            vectors,
            begin,
            end,
            len,
            num_slices,
        }
    }

    pub fn get_raw_slice(&self, s: usize) -> Option<RawSlice<T>> {
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

                Some(slice.raw_slice(start, end_exc))
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

    pub fn num_slices(&self) -> usize {
        self.num_slices
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn into_iter_ref(self) -> RawJaggedSliceIterRef<'a, T> {
        RawJaggedSliceIterRef::new(self)
    }

    pub fn into_iter_owned(self) -> RawJaggedSliceIterOwned<'a, T> {
        RawJaggedSliceIterOwned::new(self)
    }
}
