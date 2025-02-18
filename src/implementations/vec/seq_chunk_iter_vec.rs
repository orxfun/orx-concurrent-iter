use crate::implementations::ptr_utils::take;
use core::{iter::FusedIterator, marker::PhantomData};

pub struct SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    completed: bool,
    current: *const T,
    first: *const T,
    last: *const T,
    phantom: PhantomData<&'i ()>,
}

impl<'i, T> SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    pub(super) fn new(first: *const T, last: *const T) -> Self {
        Self {
            completed: first.is_null(),
            current: first,
            first,
            last,
            phantom: PhantomData,
        }
    }

    fn remaining(&self) -> usize {
        unsafe { self.last.offset_from(self.first) as usize }
    }
}

impl<'i, T> Default for SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    fn default() -> Self {
        let p: *const T = core::ptr::null();
        Self::new(p.clone(), p)
    }
}

impl<'i, T> Drop for SeqChunksIterVec<'i, T>
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
    }
}

impl<'i, T> Iterator for SeqChunksIterVec<'i, T>
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

impl<'i, T> ExactSizeIterator for SeqChunksIterVec<'i, T>
where
    T: Send + Sync,
{
    fn len(&self) -> usize {
        self.remaining()
    }
}

impl<'i, T> FusedIterator for SeqChunksIterVec<'i, T> where T: Send + Sync {}
