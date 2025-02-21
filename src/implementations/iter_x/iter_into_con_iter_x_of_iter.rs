use super::con_iter_x_of_iter::ConIterXOfIter;

pub trait IntoConIterXOfIter<T>: Iterator<Item = T> + Sized
where
    T: Send + Sync,
{
    fn into_concurrent_iter_x(self) -> ConIterXOfIter<Self, T>;
}

impl<T, I> IntoConIterXOfIter<T> for I
where
    I: Iterator<Item = T>,
    T: Send + Sync,
{
    fn into_concurrent_iter_x(self) -> ConIterXOfIter<Self, T> {
        ConIterXOfIter::new(self)
    }
}
