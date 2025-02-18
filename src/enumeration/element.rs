pub trait Element {
    type ElemOf<T>: Send + Sync
    where
        T: Send + Sync;

    type IterOf<I>;
}

pub struct Value;

impl Element for Value {
    type ElemOf<T>
        = T
    where
        T: Send + Sync;

    type IterOf<I> = I;
}

pub struct IdxValue;

impl Element for IdxValue {
    type ElemOf<T>
        = (usize, T)
    where
        T: Send + Sync;

    type IterOf<I> = (usize, I);
}
