use super::raw_jagged_owned::RawJagged;
use crate::implementations::{
    jagged_arrays::{as_slice::AsOwningSlice, jagged_indexer::JaggedIndexer},
    ptr_utils::take,
};

/// An iterator over remaining (not already taken) elements of a raw jagged array [`RawJagged`].
///
/// This iterator will take the remaining elements out of the jagged array and return.
///
/// When not all elements of the iterator are visited, this iterator prevents memory leaks by
/// dropping remaining element in place.
///
/// Furthermore, it is responsible from releasing the allocation of arrays of the jagged array on drop.
pub struct RawJaggedIterOwned<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
    jagged: RawJagged<S, T, X>,
    f: usize,
    current_ptr: *const T,
    current_last: *const T,
}

impl<S, T, X> RawJaggedIterOwned<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
    pub(crate) fn new(mut jagged: RawJagged<S, T, X>) -> Self {
        let num_taken = match jagged.num_taken() {
            Some(num_taken) => {
                // SAFETY: we assume all elements are taken out.
                // All elements will be taken out by this iterator, even if not all elements
                // are visited, Drop implementation of this iterator guarantees that all elements
                // will be dropped.
                unsafe { jagged.set_num_taken(Some(jagged.len())) };
                num_taken
            }
            // # SAFETY: when jagged.num_taken() is None, we are not responsible for and should
            // not be dropping elements; and hence, we assume all elements are taken out or
            // dropped externally.
            None => jagged.len(),
        };

        // jagged index of the first element to visit
        // SAFETY: it will only be Some pointing to position of an existing element only if not
        // all elements are already taken out.
        let taken_jagged_idx = match num_taken < jagged.len() {
            true => jagged.jagged_index(num_taken),
            false => None,
        };

        let (f, current_ptr, current_last) = match taken_jagged_idx {
            // basically an empty iterator
            None => (jagged.num_arrays(), core::ptr::null(), core::ptr::null()),
            Some(idx) => {
                let [f, i] = [idx.f, idx.i];

                // SAFETY: f is in bounds
                let vec = unsafe { jagged.get_unchecked(f) };

                let [first, last] = vec.first_and_last_ptrs();

                // SAFETY: first and last are not null pointers as vec is not empty,
                // this is guaranteed by taken_jagged_idx being of Some variant.
                // For the same reason first+i is within bounds.
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
        match self.jagged.get(self.f) {
            Some(slice) if slice.is_empty() => self.next_vec(),
            Some(slice) => {
                // SAFETY: pointers are not null since slice is not empty
                [self.current_ptr, self.current_last] = slice.first_and_last_ptrs();
                self.f += 1;
                self.next()
            }
            None => None,
        }
    }

    fn drop_next_vec(&mut self) -> bool {
        match self.jagged.get(self.f) {
            Some(slice) if slice.is_empty() => self.drop_next_vec(),
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
            true => self.drop_next_vec(),
        }
    }
}

impl<S, T, X> Iterator for RawJaggedIterOwned<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
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

                // SAFETY: ptr is valid and its value can be taken.
                // Drop will skip this position which is now uninitialized.
                Some(unsafe { take(ptr) })
            }
            true => self.next_vec(),
        }
    }
}

impl<S, T, X> Drop for RawJaggedIterOwned<S, T, X>
where
    X: JaggedIndexer,
    S: AsOwningSlice<T>,
{
    fn drop(&mut self) {
        while self.drop_next() {}
    }
}
