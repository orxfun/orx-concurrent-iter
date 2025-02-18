use crate::implementations::ptr_utils::take;
use std::iter::FusedIterator;

pub struct VecIntoSeqIter<T>
where
    T: Send + Sync,
{
    completed: bool,
    first: *const T,
    last: *const T,
    drop_vec_capacity: Option<usize>,
    current: *const T,
}

impl<T> VecIntoSeqIter<T>
where
    T: Send + Sync,
{
    pub(super) fn new(
        completed: bool,
        first: *const T,
        last: *const T,
        current: *const T,
        drop_vec_capacity: Option<usize>,
    ) -> Self {
        Self {
            completed,
            first,
            last,
            drop_vec_capacity,
            current,
        }
    }

    fn remaining(&self) -> usize {
        unsafe { self.last.offset_from(self.first) as usize }
    }
}

impl<T> Default for VecIntoSeqIter<T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        let p: *const T = core::ptr::null();
        Self::new(true, p, p, p, None)
    }
}

impl<T> Drop for VecIntoSeqIter<T>
where
    T: Send + Sync,
{
    fn drop(&mut self) {
        loop {
            match self.completed {
                false => {
                    let _ = unsafe { take(self.current as *mut T) };
                    match self.current == self.last {
                        true => self.completed = true,
                        false => self.current = unsafe { self.current.add(1) },
                    }
                }
                true => break,
            }
        }

        if let Some(vec_cap) = self.drop_vec_capacity {
            let _vec_to_drop = unsafe { Vec::from_raw_parts(self.first as *mut T, 0, vec_cap) };
        }
    }
}

impl<T> Iterator for VecIntoSeqIter<T>
where
    T: Send + Sync,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.completed {
            false => {
                let value = Some(unsafe { take(self.current as *mut T) });
                match self.current == self.last {
                    true => self.completed = true,
                    false => self.current = unsafe { self.current.add(1) },
                }
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

impl<T> ExactSizeIterator for VecIntoSeqIter<T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<T> FusedIterator for VecIntoSeqIter<T> where T: Send + Sync {}
