use super::mut_handle::MutHandle;
use crate::enumeration::{Element, Enumeration};
use core::{cell::UnsafeCell, marker::PhantomData};

pub struct IterCell<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: UnsafeCell<I>,
    num_taken: UnsafeCell<usize>,
    phantom: PhantomData<T>,
}

impl<T, I> From<I> for IterCell<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    fn from(iter: I) -> Self {
        Self {
            iter: iter.into(),
            num_taken: 0.into(),
            phantom: PhantomData,
        }
    }
}

impl<T, I> IterCell<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    pub fn into_inner(self) -> I {
        self.iter.into_inner()
    }

    /// # SAFETY
    ///
    /// Only one thread can call this method at a given instant.
    /// This is satisfied by the mut handle.
    #[inline(always)]
    pub fn next<E>(
        &self,
        mut handle: MutHandle,
    ) -> Option<<<E as Enumeration>::Element as Element>::ElemOf<T>>
    where
        E: Enumeration,
    {
        match unsafe { &mut *self.iter.get() }.next() {
            Some(item) => {
                let num_taken = unsafe { &mut *self.num_taken.get() };
                let idx = *num_taken;
                *num_taken = idx + 1;
                Some(E::new_element(idx, item))
            }
            None => {
                handle.set_target_to_completed();
                None
            }
        }
    }
}
