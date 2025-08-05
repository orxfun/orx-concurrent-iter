use crate::{ConcurrentCollection, ConcurrentIter, IntoConcurrentIter};

pub trait ConcurrentCollectionMut: ConcurrentCollection {
    type IterMut<'a>: ConcurrentIter<Item = &'a mut Self::Item>
    where
        Self: 'a;

    fn con_iter_mut(&mut self) -> Self::IterMut<'_>;
}

impl<X> ConcurrentCollectionMut for X
where
    X: IntoConcurrentIter,
    for<'a> &'a X: IntoConcurrentIter<Item = &'a <X as IntoConcurrentIter>::Item>,
    for<'a> &'a mut X: IntoConcurrentIter<Item = &'a mut <X as IntoConcurrentIter>::Item>,
{
    type IterMut<'a>
        = <&'a mut X as IntoConcurrentIter>::IntoIter
    where
        Self: 'a;

    fn con_iter_mut(&mut self) -> Self::IterMut<'_> {
        <&mut X as IntoConcurrentIter>::into_con_iter(self)
    }
}
