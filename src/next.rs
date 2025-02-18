use core::iter::Enumerate;

mod sealed {
    pub trait NextKindSealed {}
}

pub trait NextKind: sealed::NextKindSealed {
    type Next<T>;

    type SeqIterKind<I>: Default
    where
        I: Iterator + Default;

    type BeginIdx: Default + Copy;

    fn new_next<T>(begin_idx: usize, value: T) -> Self::Next<T>;

    fn destruct_next<T>(next: Self::Next<T>) -> (Self::BeginIdx, T);

    fn new_seq_iter<I: Iterator + Default>(iter: I) -> Self::SeqIterKind<I>;

    fn seq_iter_next<I: Iterator + Default>(
        begin_idx: Self::BeginIdx,
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
        = I
    where
        I: Iterator + Default;

    type BeginIdx = ();

    #[inline(always)]
    fn new_next<T>(_: usize, value: T) -> Self::Next<T> {
        value
    }

    #[inline(always)]
    fn destruct_next<T>(next: Self::Next<T>) -> (Self::BeginIdx, T) {
        ((), next)
    }

    fn new_seq_iter<I: Iterator + Default>(iter: I) -> Self::SeqIterKind<I> {
        iter
    }

    #[inline(always)]
    fn seq_iter_next<I: Iterator + Default>(
        _: Self::BeginIdx,
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
        = Enumerate<I>
    where
        I: Iterator + Default;

    type BeginIdx = usize;

    #[inline(always)]
    fn new_next<T>(begin_idx: Self::BeginIdx, value: T) -> Self::Next<T> {
        (begin_idx, value)
    }

    #[inline(always)]
    fn destruct_next<T>(next: Self::Next<T>) -> (Self::BeginIdx, T) {
        next
    }

    fn new_seq_iter<I: Iterator + Default>(iter: I) -> Self::SeqIterKind<I> {
        iter.enumerate()
    }

    #[inline(always)]
    fn seq_iter_next<I: Iterator + Default>(
        begin_idx: usize,
        seq_iter: &mut Self::SeqIterKind<I>,
    ) -> Option<Self::Next<I::Item>> {
        seq_iter.next().map(|(i, x)| (begin_idx + i, x))
    }
}
