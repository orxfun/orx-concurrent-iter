use super::raw_jagged_slice::RawJaggedSlice;

pub struct RawJaggedSliceIterOwned<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    current_last: *const T,
    current_ptr: *const T,
}

impl<'a, T> Default for RawJaggedSliceIterOwned<'a, T> {
    fn default() -> Self {
        Self {
            slice: Default::default(),
            f: Default::default(),
            current_last: core::ptr::null(),
            current_ptr: core::ptr::null(),
        }
    }
}

impl<'a, T> RawJaggedSliceIterOwned<'a, T> {
    pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        Self {
            slice,
            ..Default::default()
        }
    }

    fn next_slice(&mut self) -> Option<T> {
        match self.f == self.slice.num_slices() {
            false => {
                // let slice = self.slice.get_slice(self.f).unwrap_or(Default::default());
                // self.current = slice.iter();
                // self.f += 1;
                // self.next()
                None
            }
            true => None,
        }
    }

    // pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
    //     let f = 0;

    //     let first_slice = slice.get_slice(f).unwrap_or(Default::default());
    //     let current = first_slice.iter();
    //     Self { slice, f, current }
    // }
}

impl<'a, T> RawJaggedSliceIterOwned<'a, T> {}
