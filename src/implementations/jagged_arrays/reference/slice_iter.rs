use super::slice::RawJaggedSlice;
use crate::implementations::jagged_arrays::{JaggedIndexer, as_slice::AsSlice};

/// An iterator over references to elements of a slice of a raw jagged array;
/// i.e., a [`RawJaggedSlice`].
pub struct RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    slice: RawJaggedSlice<'a, T, S, X>,
    len_of_remaining_slices: usize,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, T, S, X> Default for RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    fn default() -> Self {
        Self {
            slice: Default::default(),
            len_of_remaining_slices: Default::default(),
            f: Default::default(),
            current: Default::default(),
        }
    }
}

impl<'a, T, S, X> RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    pub(crate) fn new(slice: RawJaggedSlice<'a, T, S, X>) -> Self {
        Self {
            len_of_remaining_slices: slice.len(),
            slice,
            f: 0,
            current: Default::default(),
        }
    }

    fn remaining(&self) -> usize {
        let remaining_current = self.current.len();
        self.len_of_remaining_slices + remaining_current
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        self.slice.get_slice(self.f).and_then(|slice| {
            match self.len_of_remaining_slices > slice.len() {
                true => {
                    self.len_of_remaining_slices -= slice.len();
                    self.f += 1;
                }
                false => {
                    self.len_of_remaining_slices = 0;
                    self.f = self.slice.num_slices();
                }
            }

            self.current = slice.iter();
            self.next()
        })
    }
}

impl<'a, T, S, X> Iterator for RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
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

impl<'a, T, S, X> ExactSizeIterator for RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}
