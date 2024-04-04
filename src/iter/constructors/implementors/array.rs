use crate::{
    iter::{constructors::into_con_iter::IntoConcurrentIter, implementors::array::ConIterOfArray},
    ConIterOfSlice, ConcurrentIterable,
};

impl<const N: usize, T: Send + Sync + Default> ConcurrentIterable for [T; N] {
    type Item<'i> = &'i T where Self: 'i;

    type ConIter<'i> = ConIterOfSlice<'i, T> where Self: 'i;

    fn con_iter(&self) -> Self::ConIter<'_> {
        Self::ConIter::new(self.as_slice())
    }
}

impl<const N: usize, T: Send + Sync + Default> IntoConcurrentIter for [T; N] {
    type Item = T;

    type ConIter = ConIterOfArray<N, T>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{iter::constructors::into_con_iter::IterIntoConcurrentIter, ConcurrentIter};

    #[test]
    fn con_iter() {
        let values = ['a', 'b', 'c'];

        let con_iter = values.con_iter();
        assert_eq!(con_iter.next(), Some(&'a'));
        assert_eq!(con_iter.next(), Some(&'b'));
        assert_eq!(con_iter.next(), Some(&'c'));
        assert_eq!(con_iter.next(), None);
    }

    #[test]
    fn into_con_iter() {
        let values = ['a', 'b', 'c'];

        let con_iter = values.into_con_iter();
        assert_eq!(con_iter.next(), Some('a'));
        assert_eq!(con_iter.next(), Some('b'));
        assert_eq!(con_iter.next(), Some('c'));
        assert_eq!(con_iter.next(), None);

        let con_iter = values.into_iter().take(2).into_con_iter();
        assert_eq!(con_iter.next(), Some('a'));
        assert_eq!(con_iter.next(), Some('b'));
        assert_eq!(con_iter.next(), None);
    }
}
