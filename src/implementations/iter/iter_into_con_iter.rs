use super::con_iter_of_iter::ConIterOfIter;

pub trait IterIntoConcurrentIter<T>: Iterator<Item = T> + Sized
where
    T: Send + Sync,
{
    fn iter_into_concurrent_iter(self) -> ConIterOfIter<Self, T>;
}

impl<T, I> IterIntoConcurrentIter<T> for I
where
    I: Iterator<Item = T>,
    T: Send + Sync,
{
    fn iter_into_concurrent_iter(self) -> ConIterOfIter<Self, T> {
        ConIterOfIter::new(self)
    }
}
