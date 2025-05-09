use super::{jagged_index::JaggedIndex, raw_slice::RawSlice};

pub struct RawJaggedSlice<'a, T> {
    slices: &'a [RawSlice<T>],
    begin: JaggedIndex,
    end: JaggedIndex,
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
    pub fn new(slices: &'a [RawSlice<T>], begin: JaggedIndex, end: JaggedIndex) -> Self {
        assert!(begin.is_in_exc_bounds_of(&slices));
        assert!(end.is_in_exc_bounds_of(&slices));
        assert!(begin <= end);
        Self { slices, begin, end }
    }

    /// Returns the `f`-th slice of the jagged slice.
    ///
    /// Returns None if `f` is out of bounds, or the corresponding slice is empty.
    /// Therefore, if this method returns Some, returned slice always have at least one element.
    pub fn get_slice(&self, s: usize) -> Option<&'a [T]> {
        // let f = self.begin[0] + s;
        // match (self.begin[0]..self.end[0]).contains(&f) {
        //     true => {
        //         let i = match f == self.begin[0] {
        //             true => self.begin[1],
        //             false => 0,
        //         };
        //         let j = match f == self.end[0] - 1 {
        //             false => self.slices[f].len(),
        //             true => match self.end[0] == self.slices.len() {
        //                 true => self.slices[f - 1].len(),
        //                 false => self.end[1],
        //             },
        //         };

        //         self.slices[f].slice(i, j - i)
        //     }
        //     false => None,
        // }
        todo!()
    }

    pub fn num_slices(&self) -> usize {
        // self.end[0] - self.begin[0]
        todo!()
    }
}
