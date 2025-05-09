use super::raw_jagged_slice::RawJaggedSlice;
use crate::implementations::ptr_utils::take;
use std::fmt::Debug;

pub struct RawJaggedSliceIterOwned<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    current_ptr: *mut T,
    current_last: *const T,
}

impl<'a, T> Default for RawJaggedSliceIterOwned<'a, T> {
    fn default() -> Self {
        Self {
            slice: Default::default(),
            f: Default::default(),
            current_ptr: core::ptr::null_mut(),
            current_last: core::ptr::null(),
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
        match self.slice.get_raw_slice(self.f) {
            Some(slice) => match slice.len() == 0 {
                true => self.next_slice(),
                false => {
                    self.current_ptr = slice.ptr();
                    self.current_last = unsafe { self.current_ptr.add(slice.len() - 1) };
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
                unsafe { self.current_ptr.drop_in_place() };

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
                let ptr = self.current_ptr;
                let is_last_of_slice = ptr as *const T == self.current_last;
                self.current_ptr = match is_last_of_slice {
                    false => unsafe { self.current_ptr.add(1) },
                    true => core::ptr::null_mut(),
                };
                Some(unsafe { take(ptr) })
            }
            true => self.next_slice(),
        }
    }
}

impl<'a, T> Drop for RawJaggedSliceIterOwned<'a, T> {
    fn drop(&mut self) {
        while self.drop_next() {}
    }
}
