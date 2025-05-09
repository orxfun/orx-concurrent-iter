use super::{
    jagged_index::JaggedIndex, raw_jagged_slice_iter_ref::RawJaggedSliceIterRef,
    raw_slice::RawSlice,
};

pub struct RawJaggedSlice<'a, T> {
    slices: &'a [RawSlice<T>],
    begin: JaggedIndex,
    end: JaggedIndex,
    num_slices: usize,
}

impl<'a, T> Default for RawJaggedSlice<'a, T> {
    fn default() -> Self {
        Self {
            slices: Default::default(),
            begin: Default::default(),
            end: Default::default(),
            num_slices: Default::default(),
        }
    }
}

impl<'a, T> RawJaggedSlice<'a, T> {
    pub fn new(slices: &'a [RawSlice<T>], begin: JaggedIndex, end: JaggedIndex) -> Self {
        assert!(begin.is_in_inc_bounds_of(&slices));
        assert!(end.is_in_exc_bounds_of(&slices));
        dbg!(&begin, &end);
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

        Self {
            slices,
            begin,
            end,
            num_slices,
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
                let slice = &self.slices[f];

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

    pub fn into_iter_ref(self) -> RawJaggedSliceIterRef<'a, T> {
        RawJaggedSliceIterRef::new(self)
    }
}
