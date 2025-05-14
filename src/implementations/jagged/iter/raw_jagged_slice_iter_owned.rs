use crate::implementations::{jagged::raw_jagged_slice::RawJaggedSlice, ptr_utils::take};

/// An iterator over owned elements of a slice of a raw jagged array;
/// i.e., a [`RawJaggedSlice`].
///
/// The elements returned by the iterator will be the elements of the jagged slice taken out.
///
/// All elements of the [`RawJaggedSlice`] that this iterator is created from will be dropped
/// in either one of the two ways:
///
/// * If the element is visited by the iterator, it will be taken out of the jagged array
///   it belongs to and will be dropped by the caller side.
/// * If not elements of the iterator are visited, an unvisited element will be dropped in
///   place while this iterator is being dropped.
pub struct RawJaggedSliceIterOwned<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    len_of_remaining_slices: usize,
    f: usize,
    current_ptr: *const T,
    current_last: *const T,
}

impl<'a, T> Default for RawJaggedSliceIterOwned<'a, T> {
    fn default() -> Self {
        Self {
            slice: Default::default(),
            len_of_remaining_slices: 0,
            f: 0,
            current_ptr: core::ptr::null(),
            current_last: core::ptr::null(),
        }
    }
}

impl<'a, T> RawJaggedSliceIterOwned<'a, T> {
    pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        Self {
            len_of_remaining_slices: slice.len(),
            slice,
            ..Default::default()
        }
    }

    fn remaining(&self) -> usize {
        let remaining_current = match self.current_ptr.is_null() {
            true => 0,
            // SAFETY: whenever current_ptr is not null, we know that current_last is also not
            // null which is >= current_ptr.
            false => unsafe { self.current_last.offset_from(self.current_ptr) as usize + 1 },
        };

        self.len_of_remaining_slices + remaining_current
    }

    fn next_slice(&mut self) -> Option<T> {
        match self.slice.get_raw_slice(self.f) {
            Some(slice) if slice.is_empty() => self.next_slice(),
            Some(slice) => {
                self.len_of_remaining_slices -= slice.len();
                // SAFETY: pointers are not null since slice is not empty
                [self.current_ptr, self.current_last] = slice.first_and_last_ptrs();
                self.f += 1;
                self.next()
            }
            None => None,
        }
    }

    fn drop_next_slice(&mut self) -> bool {
        match self.slice.get_raw_slice(self.f) {
            Some(slice) if slice.is_empty() => self.drop_next_slice(),
            Some(slice) => {
                // SAFETY: pointers are not null since slice is not empty
                [self.current_ptr, self.current_last] = slice.first_and_last_ptrs();
                self.f += 1;
                self.drop_next()
            }
            None => false,
        }
    }

    fn drop_next(&mut self) -> bool {
        match self.current_ptr.is_null() {
            false => {
                let is_last_of_slice = self.current_ptr as *const T == self.current_last;

                // SAFETY: current pointer is not null
                unsafe { (self.current_ptr as *mut T).drop_in_place() };

                self.current_ptr = match is_last_of_slice {
                    // SAFETY: current_ptr is not the last element, hance current_ptr+1 is in bounds
                    false => unsafe { self.current_ptr.add(1) },
                    true => core::ptr::null_mut(),
                };

                true
            }
            true => self.drop_next_slice(),
        }
    }
}

impl<'a, T> Iterator for RawJaggedSliceIterOwned<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_ptr.is_null() {
            false => {
                let is_last_of_slice = self.current_ptr == self.current_last;

                let ptr = self.current_ptr as *mut T;

                self.current_ptr = match is_last_of_slice {
                    // SAFETY: current_ptr is not the last element, hance current_ptr+1 is in bounds
                    false => unsafe { self.current_ptr.add(1) },
                    true => core::ptr::null_mut(),
                };

                Some(unsafe { take(ptr) })
            }
            true => self.next_slice(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining();
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for RawJaggedSliceIterOwned<'a, T> {
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<'a, T> Drop for RawJaggedSliceIterOwned<'a, T> {
    fn drop(&mut self) {
        while self.drop_next() {}
    }
}
