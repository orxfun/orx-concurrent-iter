// use crate::implementations::jagged_arrays::RawSlice;

// /// An iterator over references to elements of a slice of a raw jagged array;
// /// i.e., a [`RawJaggedSlice`].
// ///
// /// This iterator has no responsibility and does nothing about dropping elements
// /// or releasing allocated memory.
// pub struct RawJaggedSliceIterRef<'a, O, T> {
//     slice: RawJaggedSlice<O, RawSlice<T>, T>,
//     len_of_remaining_slices: usize,
//     f: usize,
//     current: core::slice::Iter<'a, T>,
// }
