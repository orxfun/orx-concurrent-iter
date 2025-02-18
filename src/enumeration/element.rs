pub trait Element {
    type ElemOf<T>;
}

pub struct Value;

impl Element for Value {
    type ElemOf<T> = T;
}

pub struct IdxValue;

impl Element for IdxValue {
    type ElemOf<T> = (usize, T);
}
