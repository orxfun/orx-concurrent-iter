use core::{fmt::Debug, iter::Enumerate};

pub(crate) mod sealed {
    pub trait NextKindSealed {}
}

pub trait NextKind: sealed::NextKindSealed + Send + Sync {
    type Next<T>;

    type SeqIterKind<I>: Default
    where
        I: Iterator + Default;

    type BeginIdx: Default + Copy + PartialEq + Debug;

    fn new_next<T>(begin_idx: usize, value: T) -> Self::Next<T>;

    fn destruct_next<T>(next: Self::Next<T>) -> (Self::BeginIdx, T);

    fn new_seq_iter<I: Iterator + Default>(iter: I) -> Self::SeqIterKind<I>;

    fn seq_iter_next<I: Iterator + Default>(
        begin_idx: Self::BeginIdx,
        seq_iter: &mut Self::SeqIterKind<I>,
    ) -> Option<Self::Next<I::Item>>;

    #[cfg(test)]
    fn construct_next<T>(begin_idx: Self::BeginIdx, value: T) -> Self::Next<T>;

    #[cfg(test)]
    fn eq_next<T: PartialEq>(a: Self::Next<T>, b: Self::Next<T>) -> bool {
        let (a1, a2) = Self::destruct_next(a);
        let (b1, b2) = Self::destruct_next(b);
        a1 == b1 && a2 == b2
    }

    #[cfg(test)]
    fn validate_begin_idx(begin_idx: Self::BeginIdx, validate: impl Fn(usize) -> bool) -> bool;

    #[cfg(test)]
    fn map_begin_idx(begin_idx: Self::BeginIdx, map: impl Fn(usize) -> usize) -> Self::BeginIdx;
}

#[derive(Default)]
pub struct Regular;

impl sealed::NextKindSealed for Regular {
    //
}

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

    #[cfg(test)]
    fn construct_next<T>(_: Self::BeginIdx, value: T) -> Self::Next<T> {
        value
    }

    #[cfg(test)]
    fn validate_begin_idx(_: Self::BeginIdx, _: impl Fn(usize) -> bool) -> bool {
        true
    }

    #[cfg(test)]
    fn map_begin_idx(_: Self::BeginIdx, _: impl Fn(usize) -> usize) -> Self::BeginIdx {
        ()
    }
}

#[derive(Default)]
pub struct Enumerated;

impl sealed::NextKindSealed for Enumerated {
    //
}

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

    #[cfg(test)]
    fn construct_next<T>(begin_idx: Self::BeginIdx, value: T) -> Self::Next<T> {
        (begin_idx, value)
    }

    #[cfg(test)]
    fn validate_begin_idx(begin_idx: Self::BeginIdx, validate: impl Fn(usize) -> bool) -> bool {
        validate(begin_idx)
    }

    #[cfg(test)]
    fn map_begin_idx(begin_idx: Self::BeginIdx, map: impl Fn(usize) -> usize) -> Self::BeginIdx {
        map(begin_idx)
    }
}
