use crate::implementations::ptr_utils::take;
use alloc::boxed::Box;
use core::iter::FusedIterator;

/// A partially-taken contagious memory converted into sequential iterator which makes sure that
/// either all elements will be taken and returned by the iterator or will be dropped once the
/// iterator is dropped.
///
/// Further, it might call a `drop_allocation` function if provided to drop the vector.
pub struct ArrayIntoSeqIter<T>
where
    T: Send + Sync,
{
    current: *const T,
    last: *const T,
    drop_allocation: Option<Box<dyn Fn()>>,
}

impl<T> ArrayIntoSeqIter<T>
where
    T: Send + Sync,
{
    /// Creates the new iterator such that:
    /// * the first element to be returned is at `first` (inclusive)
    /// * the last element of allocation is at `last` (inclusive)
    /// * when Some(drop_allocation) is provided, it will be called while dropping
    ///   this iterator to clean up allocation, if required.
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
    /// * further, all addresses between `first` and `last` are valid pointers to the same contiguous allocation.
    ///
    /// Finally, when provided, `drop_allocation` is expected to drop the allocation for the completely or partially
    /// given contiguous memory; however, not the elements. This iterator will make sure that all elements between
    /// `first` and `last` are dropped, regardless of whether the iterator is completely traversed or not.
    pub(super) fn new(
        first: *const T,
        last: *const T,
        drop_allocation: Option<Box<dyn Fn()>>,
    ) -> Self {
        Self {
            current: first,
            last,
            drop_allocation,
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

impl<T> Default for ArrayIntoSeqIter<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        let p: *const T = core::ptr::null();
        Self::new(p, p, None)
    }
}

impl<T> Drop for ArrayIntoSeqIter<T>
where
    T: Send + Sync,
{
    fn drop(&mut self) {
        while !self.current.is_null() {
            // SAFETY: p is valid, not yat taken out or dropped
            let p = self.current as *mut T;
            unsafe { p.drop_in_place() };

            let completed = self.current == self.last;
            self.current = match completed {
                true => core::ptr::null(),
                // SAFETY: since current is not yet last, and since last is valid and inclusive, it is safe to add(1)
                false => unsafe { self.current.add(1) },
            };
        }

        if let Some(drop_allocation) = &self.drop_allocation {
            drop_allocation();
        }
    }
}

impl<T> Iterator for ArrayIntoSeqIter<T>
where
    T: Send + Sync,
{
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

impl<T> ExactSizeIterator for ArrayIntoSeqIter<T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<T> FusedIterator for ArrayIntoSeqIter<T> where T: Send + Sync {}
