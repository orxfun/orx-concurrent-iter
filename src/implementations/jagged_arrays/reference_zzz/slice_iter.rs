use crate::implementations::jagged_arrays::{RawSlice, jagged_slice2::RawJaggedSlice};
use orx_self_or::SoR;

/// An iterator over references to elements of a slice of a raw jagged array;
/// i.e., a [`RawJaggedSlice`].
///
/// This iterator has no responsibility and does nothing about dropping elements
/// or releasing allocated memory.
pub struct RawJaggedSliceIterRef<'a, O, T>
where
    O: SoR<Vec<RawSlice<T>>>,
{
    slice: RawJaggedSlice<O, RawSlice<T>, T>,
    len_of_remaining_slices: usize,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, O, T> Default for RawJaggedSliceIterRef<'a, O, T>
where
    O: SoR<Vec<RawSlice<T>>>,
{
    fn default() -> Self {
        todo!()
    }
}

impl<'a, O, T> RawJaggedSliceIterRef<'a, O, T>
where
    O: SoR<Vec<RawSlice<T>>>,
{
    pub(crate) fn new(slice: RawJaggedSlice<O, RawSlice<T>, T>) -> Self {
        Self {
            len_of_remaining_slices: slice.len(),
            f: 0,
            slice,
            current: Default::default(),
        }
    }

    fn remaining(&self) -> usize {
        let remaining_current = self.current.len();
        self.len_of_remaining_slices + remaining_current
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        // SAFETY: slice returned by get_slice cannot outlive &self which cannot outlive the data owner jagged array
        unsafe { self.slice.get_slice(self.f) }.and_then(|slice| {
            self.len_of_remaining_slices -= slice.len();
            self.f += 1;
            self.next()
        })
    }
}

impl<'a, O, T> Iterator for RawJaggedSliceIterRef<'a, O, T>
where
    O: SoR<Vec<RawSlice<T>>>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next();
        match next.is_some() {
            true => next,
            false => self.next_slice(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining();
        (len, Some(len))
    }
}

impl<'a, O, T> ExactSizeIterator for RawJaggedSliceIterRef<'a, O, T>
where
    O: SoR<Vec<RawSlice<T>>>,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}
