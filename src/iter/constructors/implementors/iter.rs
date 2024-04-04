use crate::{iter::constructors::into_con_iter::IterIntoConcurrentIter, ConIterOfIter};

impl<T: Send + Sync, Iter> IterIntoConcurrentIter for Iter
where
    Iter: Iterator<Item = T>,
{
    type Item = T;

    type ConIter = ConIterOfIter<T, Iter>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConcurrentIter;

    #[test]
    fn con_iter() {
        let values = ['a', 'b', 'c'];

        let con_iter = values.iter().into_con_iter();
        assert_eq!(con_iter.next(), Some(&'a'));
        assert_eq!(con_iter.next(), Some(&'b'));
        assert_eq!(con_iter.next(), Some(&'c'));
        assert_eq!(con_iter.next(), None);

        let con_iter = values.iter().take(2).into_con_iter();
        assert_eq!(con_iter.next(), Some(&'a'));
        assert_eq!(con_iter.next(), Some(&'b'));
        assert_eq!(con_iter.next(), None);

        let con_iter = values.iter().skip(1).into_con_iter();
        assert_eq!(con_iter.next(), Some(&'b'));
        assert_eq!(con_iter.next(), Some(&'c'));
        assert_eq!(con_iter.next(), None);

        let con_iter = values
            .iter()
            .filter(|x| **x != 'a')
            .map(|x| x.to_string())
            .into_con_iter();
        assert_eq!(con_iter.next(), Some(String::from('b')));
        assert_eq!(con_iter.next(), Some(String::from('c')));
        assert_eq!(con_iter.next(), None);
    }
}
