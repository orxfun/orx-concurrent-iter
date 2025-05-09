use super::raw_slice::RawSlice;

pub struct RawJaggedSlice<'a, T> {
    slices: &'a [RawSlice<T>],
    begin: [usize; 2],
    end: [usize; 2],
}

impl<'a, T> Default for RawJaggedSlice<'a, T> {
    fn default() -> Self {
        Self {
            slices: Default::default(),
            begin: Default::default(),
            end: Default::default(),
        }
    }
}

impl<'a, T> RawJaggedSlice<'a, T> {
    pub fn new(slices: &'a [RawSlice<T>], begin: [usize; 2], end: [usize; 2]) -> Self {
        assert!(end[0] >= begin[0]);
        assert!(end[0] != begin[0] || end[1] >= begin[1]);
        assert!(begin[0] <= slices.len());
        assert!((begin[0] == slices.len() && begin[1] == 0) || begin[1] <= slices[begin[0]].len());
        assert!(end[0] <= slices.len());
        assert!((end[0] == slices.len() && end[1] == 0) || end[1] <= slices[end[0]].len());
        Self { slices, begin, end }
    }

    /// Returns the `f`-th slice of the jagged slice.
    ///
    /// Returns None if `f` is out of bounds, or the corresponding slice is empty.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub fn get_slice(&self, f: usize) -> Option<&'a [T]> {
        match (self.begin[0]..self.end[0]).contains(&f) {
            true => {
                let f = self.begin[0] + f;
                let i = match f == self.begin[0] {
                    true => self.begin[1],
                    false => 0,
                };
                self.slices[f].slice_from(i)
            }
            false => None,
        }
    }

    pub fn num_slices(&self) -> usize {
        self.end[0] - self.begin[0]
    }
}
