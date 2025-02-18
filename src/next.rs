pub(crate) trait NextKindCore {}

pub trait NextKind: NextKindCore {
    type Next<T>: Send + Sync
    where
        T: Send + Sync;

    type NextChunk<T, I>
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;
}

pub struct Regular;
impl NextKindCore for Regular {}
impl NextKind for Regular {
    type Next<T: Send + Sync> = T;

    type NextChunk<T, I>
        = I
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;
}

pub struct Enumerated;
impl NextKindCore for Enumerated {}
impl NextKind for Enumerated {
    type Next<T: Send + Sync> = (usize, T);

    type NextChunk<T, I>
        = (usize, I)
    where
        T: Send + Sync,
        I: ExactSizeIterator<Item = T>;
}
