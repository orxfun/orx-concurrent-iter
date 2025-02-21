use core::{cell::UnsafeCell, marker::PhantomData};

pub struct MutIter<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: UnsafeCell<I>,
    phantom: PhantomData<T>,
}

impl<T, I> From<I> for MutIter<T, I>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    fn from(iter: I) -> Self {
        Self {
            iter: iter.into(),
            phantom: PhantomData,
        }
    }
}
