use super::slice::RawJaggedSlice;
use crate::implementations::jagged_arrays::{JaggedIndexer, Slices};

/// An iterator over references to elements of a slice of a raw jagged array;
/// i.e., a [`RawJaggedSlice`].
pub struct RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    slice: RawJaggedSlice<'a, T, S, X>,
    len_of_remaining_slices: usize,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, T, S, X> Default for RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
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
    S: Slices<'a, T>,
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

    fn progress_to_next_slice(&mut self) -> bool {
        match self.slice.get_slice(self.f) {
            None => false,
            Some(slice) => {
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
                true
            }
        }
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        match self.progress_to_next_slice() {
            true => self.next(),
            false => None,
        }
    }
}

impl<'a, T, S, X> Iterator for RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
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

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = self.current.fold(init, &mut f);

        acc
    }
}

impl<'a, T, S, X> ExactSizeIterator for RawJaggedSliceIterRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}
