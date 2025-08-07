use crate::implementations::ptr_utils::take;
use core::iter::FusedIterator;

/// A partially-taken contagious memory converted into sequential iterator which makes sure that
/// either all elements will be taken and returned by the iterator or will be dropped once the
/// iterator is dropped.
///
/// Further, it might call a `drop_allocation` function if provided to drop the vector.
pub struct ArrayIntoSeqIter<T, M> {
    current: *const T,
    last: *const T,
    allocation_to_drop: Option<(*const T, usize)>,
    /// Data moved into the sequential iterator to make sure that it is not dropped before the iterator is dropped.
    _moved_into: M,
}

impl<T, M> ArrayIntoSeqIter<T, M> {
    /// Creates the new iterator such that:
    /// * the first element to be returned is at `first` (inclusive)
    /// * the last element of allocation is at `last` (inclusive)
    /// * when Some(allocation_to_drop) is provided, the contiguous memory starting from
    ///   the given ptr and capacity will be dropped when this iterator is dropped.
    ///
    /// An iterator with only one element can be created by providing `first = last`.
    ///
    /// An empty iterator can be created by providing `first = last = ptr::null()`;
    /// alternatively default constructor can be used.
    ///
    /// # SAFETY
    ///
    /// The caller must ensure that:
    /// * `first` and `last` are valid pointers,
    /// * further, all addresses between `first` and `last` are valid pointers to the same contiguous allocation,
    /// * when `allocation_to_drop` is provided with pointer, say `p`, all elements within `p..first` must already
    ///   be taken out or dropped.
    ///
    /// Finally, when provided, `allocation_to_drop` is expected to drop the allocation for the completely or partially
    /// given contiguous memory; however, not the elements. This iterator will make sure that all elements between
    /// `first` and `last` are dropped, regardless of whether the iterator is completely traversed or not.
    pub(crate) fn new(
        first: *const T,
        last: *const T,
        allocation_to_drop: Option<(*const T, usize)>,
        moved_into: M,
    ) -> Self {
        Self {
            current: first,
            last,
            allocation_to_drop,
            _moved_into: moved_into,
        }
    }

    fn remaining(&self) -> usize {
        match self.current.is_null() {
            // no more elements
            true => 0,
            // SAFETY: current has not yet reached last; neither last nor current are null; count is offset+1 since last is inclusive
            false => unsafe { self.last.offset_from(self.current) as usize + 1 },
        }
    }
}

impl<T, M> Default for ArrayIntoSeqIter<T, M>
where
    M: Default,
{
    fn default() -> Self {
        Self::new(core::ptr::null(), core::ptr::null(), None, M::default())
    }
}

impl<T, M> Drop for ArrayIntoSeqIter<T, M> {
    fn drop(&mut self) {
        // 1. drop remaining elements in place
        if core::mem::needs_drop::<T>() {
            while !self.current.is_null() {
                // SAFETY: p is valid, not yat taken out or dropped
                let p = self.current as *mut T;
                unsafe { p.drop_in_place() };

                let completed = self.current == self.last;
                self.current = match completed {
                    true => core::ptr::null(),
                    // SAFETY: since current has not yet reached last,
                    // and since last is valid and inclusive, it is safe to add(1)
                    false => unsafe { self.current.add(1) },
                };
            }
        }

        // 2. drop allocation
        if let Some((ptr, capacity)) = &self.allocation_to_drop {
            let _vec_to_drop =
                unsafe { alloc::vec::Vec::from_raw_parts(*ptr as *mut T, 0, *capacity) };
        }
    }
}

impl<T, M> Iterator for ArrayIntoSeqIter<T, M> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.is_null() {
            false => {
                // SAFETY: current is valid, not yat taken out or dropped
                let value = Some(unsafe { take(self.current as *mut T) });
                let completed = self.current == self.last;
                self.current = match completed {
                    true => core::ptr::null(),
                    // SAFETY: since current is not yet last, and since last is valid and inclusive, it is safe to add(1)
                    false => unsafe { self.current.add(1) },
                };
                value
            }
            true => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining();
        (len, Some(len))
    }
}

impl<T, M> ExactSizeIterator for ArrayIntoSeqIter<T, M> {
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<T, M> FusedIterator for ArrayIntoSeqIter<T, M> {}
