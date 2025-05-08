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
}
