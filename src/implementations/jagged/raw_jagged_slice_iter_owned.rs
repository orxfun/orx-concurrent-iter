use std::fmt::Debug;

use super::raw_jagged_slice::RawJaggedSlice;
use crate::implementations::ptr_utils::{read, take};

pub struct RawJaggedSliceIterOwned<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    current_ptr: *const T,
    current_last: *const T,
}

impl<'a, T> Default for RawJaggedSliceIterOwned<'a, T> {
    fn default() -> Self {
        Self {
            slice: Default::default(),
            f: Default::default(),
            current_ptr: core::ptr::null(),
            current_last: core::ptr::null(),
        }
    }
}

impl<'a, T> RawJaggedSliceIterOwned<'a, T>
where
    T: Debug,
{
    pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        Self {
            slice,
            ..Default::default()
        }
    }

    fn next_slice(&mut self) -> Option<T> {
        match self.slice.get_slice(self.f) {
            Some(slice) => match slice.is_empty() {
                true => self.next_slice(),
                false => {
                    self.current_ptr = slice.as_ptr();
                    self.current_last = unsafe { self.current_ptr.add(slice.len() - 1) };
                    self.f += 1;
                    self.next()
                }
            },
            None => None,
        }
    }
}

impl<'a, T> Iterator for RawJaggedSliceIterOwned<'a, T>
where
    T: Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_ptr.is_null() {
            false => {
                let ptr = self.current_ptr as *mut T;
                let value = Some(unsafe { take(ptr) });
                self.current_ptr = match self.current_ptr == self.current_last {
                    false => unsafe { self.current_ptr.add(1) },
                    true => core::ptr::null(),
                };
                value
            }
            true => self.next_slice(),
        }
    }
}
