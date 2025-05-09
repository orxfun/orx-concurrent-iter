use super::{raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};

pub struct RawJaggedSliceIterRef<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    current: core::slice::Iter<'a, T>,
}

// impl<'a, T> RawJaggedSliceIterRef<'a, T> {
//     pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
//         Self { slice }
//     }
// }

impl<'a, T> Iterator for RawJaggedSliceIterRef<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
