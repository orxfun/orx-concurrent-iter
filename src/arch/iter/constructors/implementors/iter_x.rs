use crate::iter::{
    constructors::into_con_iter_x::IntoConcurrentIterX, implementors::iter_x::ConIterOfIterX,
};

impl<T: Send + Sync, Iter> IntoConcurrentIterX for Iter
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    type ConIter = ConIterOfIterX<T, Iter>;

    fn into_con_iter_x(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}
