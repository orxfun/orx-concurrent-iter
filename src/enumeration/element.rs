pub trait Element {
    type ElemOf<T>: Send + Sync
    where
        T: Send + Sync;

    type IterOf<I>;

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T;

    fn cloned_elem<T: Send + Sync + Clone>(elem: Self::ElemOf<&T>) -> Self::ElemOf<T>;
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

    fn cloned_elem<T: Send + Sync + Clone>(elem: Self::ElemOf<&T>) -> Self::ElemOf<T> {
        elem.clone()
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

    fn cloned_elem<T: Send + Sync + Clone>(elem: Self::ElemOf<&T>) -> Self::ElemOf<T> {
        (elem.0, elem.1.clone())
    }
}
