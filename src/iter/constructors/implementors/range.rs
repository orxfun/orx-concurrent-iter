use crate::{
    iter::{
        constructors::{
            into_con_iter::IntoConcurrentIter, into_exact_con_iter::IntoExactSizeConcurrentIter,
        },
        implementors::range::ConIterOfRange,
    },
    ConcurrentIterable,
};
use std::ops::{Add, Range, Sub};

impl<Idx> ConcurrentIterable for Range<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
{
    type Item<'i> = Idx where Self: 'i;

    type ConIter<'i> = ConIterOfRange<Idx> where Self: 'i;

    fn con_iter(&self) -> Self::ConIter<'_> {
        Self::ConIter::new(self.clone())
    }
}

impl<Idx> IntoConcurrentIter for Range<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
{
    type Item = Idx;

    type ConIter = ConIterOfRange<Idx>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}

impl<Idx> IntoExactSizeConcurrentIter for Range<Idx>
where
    Idx: Send
        + Sync
        + Clone
        + Copy
        + From<usize>
        + Into<usize>
        + Add<Idx, Output = Idx>
        + Sub<Idx, Output = Idx>
        + Ord,
{
    type Item = Idx;

    type ExactConIter = ConIterOfRange<Idx>;

    fn into_exact_con_iter(self) -> Self::ExactConIter {
        Self::ExactConIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConIterOfIter, ConcurrentIter};

    #[test]
    fn con_iter() {
        let values = 42..45;

        let con_iter: ConIterOfRange<_> = values.con_iter();
        assert_eq!(con_iter.next(), Some(42));
        assert_eq!(con_iter.next(), Some(43));
        assert_eq!(con_iter.next(), Some(44));
        assert_eq!(con_iter.next(), None);

        let con_iter: ConIterOfRange<_> = values.clone().into_con_iter();
        assert_eq!(con_iter.next(), Some(42));
        assert_eq!(con_iter.next(), Some(43));
        assert_eq!(con_iter.next(), Some(44));
        assert_eq!(con_iter.next(), None);

        let con_iter: ConIterOfRange<_> = values.into_exact_con_iter();
        assert_eq!(con_iter.next(), Some(42));
        assert_eq!(con_iter.next(), Some(43));
        assert_eq!(con_iter.next(), Some(44));
        assert_eq!(con_iter.next(), None);
    }

    #[test]
    fn into_con_iter() {
        use crate::IterIntoConcurrentIter;

        let values = 42..45;

        let con_iter: ConIterOfIter<_, _> = values.into_iter().take(2).into_con_iter();
        assert_eq!(con_iter.next(), Some(42));
        assert_eq!(con_iter.next(), Some(43));
        assert_eq!(con_iter.next(), None);
    }
}
