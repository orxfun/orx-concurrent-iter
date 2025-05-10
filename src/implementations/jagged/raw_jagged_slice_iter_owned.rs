use super::raw_jagged_slice::RawJaggedSlice;
use crate::implementations::ptr_utils::take;

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
            current_ptr: core::ptr::null_mut(),
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
            false => unsafe { self.current_last.offset_from(self.current_ptr) as usize + 1 },
        };

        self.len_of_remaining_slices + remaining_current
    }

    fn next_slice(&mut self) -> Option<T> {
        match self.slice.get_raw_slice(self.f) {
            Some(slice) => match slice.len() == 0 {
                true => self.next_slice(),
                false => {
                    self.len_of_remaining_slices -= slice.len();
                    [self.current_ptr, self.current_last] = slice.first_and_last_ptrs();
                    self.f += 1;
                    self.next()
                }
            },
            None => None,
        }
    }

    fn drop_next_slice(&mut self) -> bool {
        match self.slice.get_raw_slice(self.f) {
            Some(slice) => match slice.len() == 0 {
                true => self.drop_next_slice(),
                false => {
                    self.current_ptr = slice.ptr();
                    self.current_last = unsafe { self.current_ptr.add(slice.len() - 1) };
                    self.f += 1;
                    self.drop_next()
                }
            },
            None => false,
        }
    }

    fn drop_next(&mut self) -> bool {
        match self.current_ptr.is_null() {
            false => {
                unsafe { (self.current_ptr as *mut T).drop_in_place() };

                let is_last_of_slice = self.current_ptr as *const T == self.current_last;
                self.current_ptr = match is_last_of_slice {
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
                let ptr = self.current_ptr as *mut T;
                let is_last_of_slice = self.current_ptr == self.current_last;
                self.current_ptr = match is_last_of_slice {
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
