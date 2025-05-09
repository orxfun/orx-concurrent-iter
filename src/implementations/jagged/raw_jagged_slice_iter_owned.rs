use super::raw_jagged_slice::RawJaggedSlice;

pub struct RawJaggedSliceIterOwned<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    completed: bool,
    current_last: *const T,
    current_ptr: *const T,
}

impl<'a, T> RawJaggedSliceIterOwned<'a, T> {
    // pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
    //     let f = 0;

    //     let first_slice = slice.get_slice(f).unwrap_or(Default::default());
    //     let current = first_slice.iter();
    //     Self { slice, f, current }
    // }
}
