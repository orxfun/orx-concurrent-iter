mod sealed {
    pub trait NextKindSealed {}
}

pub trait NextKind: sealed::NextKindSealed {
    type Next<T>;

    fn new_next<T>(begin_idx: usize, value: T) -> Self::Next<T>;
}

pub struct Regular;

impl sealed::NextKindSealed for Regular {}

impl NextKind for Regular {
    type Next<T> = T;

    #[inline(always)]
    fn new_next<T>(_: usize, value: T) -> Self::Next<T> {
        value
    }
}

pub struct Enumerated;

impl sealed::NextKindSealed for Enumerated {}

impl NextKind for Enumerated {
    type Next<T> = (usize, T);

    #[inline(always)]
    fn new_next<T>(begin_idx: usize, value: T) -> Self::Next<T> {
        (begin_idx, value)
    }
}
