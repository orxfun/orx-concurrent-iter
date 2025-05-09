use super::raw_slice::RawSlice;

pub struct RawJaggedSlice<'a, T> {
    slices: &'a [RawSlice<T>],
    begin: [usize; 2],
    end: [usize; 2],
}

impl<'a, T> RawJaggedSlice<'a, T> {
    pub(super) fn new(slices: &'a [RawSlice<T>], begin: [usize; 2], end: [usize; 2]) -> Self {
        Self { slices, begin, end }
    }

    /// Returns the `f`-th slice of the jagged slice.
    ///
    /// Returns None if `f` is out of bounds, or the corresponding slice is empty.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub(super) fn get_slice(&self, f: usize) -> Option<&'a [T]> {
        match (self.begin[0]..=self.end[0]).contains(&f) {
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
}
