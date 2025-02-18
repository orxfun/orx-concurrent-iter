pub trait Element {
    type ElemOf<T>: Send + Sync
    where
        T: Send + Sync;

    type IterOf<I>;

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T;
}

pub struct Value;

impl Element for Value {
    type ElemOf<T>
        = T
    where
        T: Send + Sync;

    type IterOf<I> = I;

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T {
        element
    }
}

pub struct IdxValue;

impl Element for IdxValue {
    type ElemOf<T>
        = (usize, T)
    where
        T: Send + Sync;

    type IterOf<I> = (usize, I);

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T {
        element.1
    }
}
