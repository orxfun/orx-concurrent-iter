use super::raw_jagged::RawJagged;
use crate::implementations::ptr_utils::take;

pub struct RawJaggedIterOwned<T, X>
where
    X: Fn(usize) -> [usize; 2] + Clone,
{
    jagged: RawJagged<T, X>,
    f: usize,
    current_ptr: *const T,
    current_last: *const T,
}

impl<T, X> RawJaggedIterOwned<T, X>
where
    X: Fn(usize) -> [usize; 2] + Clone,
{
    pub(super) fn new(jagged: RawJagged<T, X>, num_taken: usize) -> Self {
        let taken_idx = match num_taken < jagged.len() {
            true => jagged.jagged_index(num_taken),
            false => None,
        };

        let (f, current_ptr, current_last) = match taken_idx {
            None => (jagged.num_slices(), core::ptr::null(), core::ptr::null()),
            Some(idx) => {
                let [f, i] = [idx.f, idx.i];
                let vec = &jagged.vectors()[f];
                let [first, last] = vec.first_and_last_ptrs();
                let current = unsafe { first.add(i) };
                (f + 1 /* next vec idx */, current, last)
            }
        };
        Self {
            jagged,
            f,
            current_ptr,
            current_last,
        }
    }

    fn next_vec(&mut self) -> Option<T> {
        match self.jagged.get_raw_slice(self.f) {
            Some(slice) => match slice.len() == 0 {
                true => self.next_vec(),
                false => {
                    [self.current_ptr, self.current_last] = slice.first_and_last_ptrs();
                    self.f += 1;
                    self.next()
                }
            },
            None => None,
        }
    }

    fn drop_next_vec(&mut self) -> bool {
        match self.jagged.get_raw_slice(self.f) {
            Some(slice) => match slice.len() == 0 {
                true => self.drop_next_vec(),
                false => {
                    [self.current_ptr, self.current_last] = slice.first_and_last_ptrs();
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
            true => self.drop_next_vec(),
        }
    }
}

impl<T, X> Iterator for RawJaggedIterOwned<T, X>
where
    X: Fn(usize) -> [usize; 2] + Clone,
{
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
            true => self.next_vec(),
        }
    }
}

impl<T, X> Drop for RawJaggedIterOwned<T, X>
where
    X: Fn(usize) -> [usize; 2] + Clone,
{
    fn drop(&mut self) {
        while self.drop_next() {}
    }
}
