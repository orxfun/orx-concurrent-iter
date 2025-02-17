use crate::{
    iter::{constructors::into_con_iter::IntoConcurrentIter, implementors::range::ConIterOfRange},
    ConcurrentIterable,
};
use core::ops::{Add, Range, Sub};

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
    Range<Idx>: Iterator<Item = Idx>,
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
    Range<Idx>: Iterator<Item = Idx>,
{
    type Item = Idx;

    type ConIter = ConIterOfRange<Idx>;

    fn into_con_iter(self) -> Self::ConIter {
        Self::ConIter::new(self)
    }
}
