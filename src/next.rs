pub(crate) trait NextKindCore {}

pub trait NextKind: NextKindCore {
    type Next<T>: Send + Sync
    where
        T: Send + Sync;

    type NextChunk<I, T>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = Self::Next<T>>;
}

pub struct Regular;
impl NextKindCore for Regular {}
impl NextKind for Regular {
    type Next<T: Send + Sync> = T;

    type NextChunk<I, T>
        = I
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = Self::Next<T>>;
}

pub struct Enumerated;
impl NextKindCore for Enumerated {}
impl NextKind for Enumerated {
    type Next<T: Send + Sync> = (usize, T);

    type NextChunk<I, T>
        = (usize, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = Self::Next<T>>;
}
