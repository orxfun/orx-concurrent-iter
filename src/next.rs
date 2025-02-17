use core::iter::Enumerate;

mod sealed {
    pub trait NextKindSealed {}
}

pub trait NextKind: sealed::NextKindSealed {
    type Next<T>;

    type SeqIterKind<I>
    where
        I: Iterator;

    fn new_next<T>(begin_idx: usize, value: T) -> Self::Next<T>;

    fn new_seq_iter<I: Iterator>(iter: I) -> Self::SeqIterKind<I>;

    fn seq_iter_next<I: Iterator>(
        begin_idx: usize,
        seq_iter: &mut Self::SeqIterKind<I>,
    ) -> Option<Self::Next<I::Item>> {
        None
    }
}

#[derive(Default)]
pub struct Regular;

impl sealed::NextKindSealed for Regular {}

impl NextKind for Regular {
    type Next<T> = T;

    type SeqIterKind<I>
    where
        I: Iterator,
    = I;

    #[inline(always)]
    fn new_next<T>(_: usize, value: T) -> Self::Next<T> {
        value
    }

    fn new_seq_iter<I: Iterator>(iter: I) -> Self::SeqIterKind<I> {
        iter
    }

    #[inline(always)]
    fn seq_iter_next<I: Iterator>(
        _: usize,
        seq_iter: &mut Self::SeqIterKind<I>,
    ) -> Option<Self::Next<I::Item>> {
        seq_iter.next()
    }
}

#[derive(Default)]
pub struct Enumerated;

impl sealed::NextKindSealed for Enumerated {}

impl NextKind for Enumerated {
    type Next<T> = (usize, T);

    type SeqIterKind<I>
    where
        I: Iterator,
    = Enumerate<I>;

    #[inline(always)]
    fn new_next<T>(begin_idx: usize, value: T) -> Self::Next<T> {
        (begin_idx, value)
    }

    fn new_seq_iter<I: Iterator>(iter: I) -> Self::SeqIterKind<I> {
        iter.enumerate()
    }

    #[inline(always)]
    fn seq_iter_next<I: Iterator>(
        begin_idx: usize,
        seq_iter: &mut Self::SeqIterKind<I>,
    ) -> Option<Self::Next<I::Item>> {
        seq_iter.next().map(|(i, x)| (begin_idx + i, x))
    }
}
