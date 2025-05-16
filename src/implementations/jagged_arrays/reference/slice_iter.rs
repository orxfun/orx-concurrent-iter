use super::slice::RawJaggedSlice;

/// An iterator over references to elements of a slice of a raw jagged array;
/// i.e., a [`RawJaggedSlice`].
pub struct RawJaggedSliceIterRef<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    len_of_remaining_slices: usize,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, T> Default for RawJaggedSliceIterRef<'a, T> {
    fn default() -> Self {
        Self {
            slice: Default::default(),
            len_of_remaining_slices: 0,
            f: 0,
            current: Default::default(),
        }
    }
}

impl<'a, T> RawJaggedSliceIterRef<'a, T> {
    pub(crate) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        Self {
            len_of_remaining_slices: slice.len(),
            slice,
            ..Default::default()
        }
    }

    fn remaining(&self) -> usize {
        let remaining_current = self.current.len();
        self.len_of_remaining_slices + remaining_current
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        unsafe { self.slice.get_slice(self.f) }.and_then(|slice| {
            self.len_of_remaining_slices -= slice.len();
            self.current = slice.iter();
            self.f += 1;
            self.next()
        })
    }
}

impl<'a, T> Iterator for RawJaggedSliceIterRef<'a, T> {
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

impl<'a, T> ExactSizeIterator for RawJaggedSliceIterRef<'a, T> {
    fn len(&self) -> usize {
        self.remaining()
    }
}
