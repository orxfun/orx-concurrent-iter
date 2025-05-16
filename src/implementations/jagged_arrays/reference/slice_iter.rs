use super::{as_raw_jagged_ref::AsRawJaggedRef, slice::RawJaggedSlice};
use crate::implementations::jagged_arrays::JaggedIndexer;

/// An iterator over references to elements of a slice of a raw jagged array;
/// i.e., a [`RawJaggedSlice`].
pub struct RawJaggedSliceIterRef<'a, J, X, T>
where
    X: JaggedIndexer,
    J: AsRawJaggedRef<'a, T, X>,
{
    slice: RawJaggedSlice<'a, J, X, T>,
    len_of_remaining_slices: usize,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

// impl<'a, J, X, T> Default for RawJaggedSliceIterRef<'a, J, X, T>
// where
//     X: JaggedIndexer,
//     J: AsRawJaggedRef<'a, T, X>,
// {
//     fn default() -> Self {
//         Self {
//             slice: Default::default(),
//             len_of_remaining_slices: 0,
//             f: 0,
//             current: Default::default(),
//         }
//     }
// }

impl<'a, J, X, T> RawJaggedSliceIterRef<'a, J, X, T>
where
    X: JaggedIndexer,
    J: AsRawJaggedRef<'a, T, X>,
{
    pub(crate) fn new(slice: RawJaggedSlice<'a, J, X, T>) -> Self
    where
        X: JaggedIndexer,
        J: AsRawJaggedRef<'a, T, X>,
    {
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
            self.len_of_remaining_slices -= slice.len();
            self.current = slice.iter();
            self.f += 1;
            self.next()
        })
    }
}

impl<'a, J, X, T> Iterator for RawJaggedSliceIterRef<'a, J, X, T>
where
    X: JaggedIndexer,
    J: AsRawJaggedRef<'a, T, X>,
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

impl<'a, J, X, T> ExactSizeIterator for RawJaggedSliceIterRef<'a, J, X, T>
where
    X: JaggedIndexer,
    J: AsRawJaggedRef<'a, T, X>,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}
