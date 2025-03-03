use core::iter::{Cloned, Copied};

pub trait Element {
    type ElemOf<T>: Send + Sync
    where
        T: Send + Sync;

    type IterOf<I>
    where
        I: Iterator;

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T;

    fn cloned_elem<T: Send + Sync + Clone>(elem: Self::ElemOf<&T>) -> Self::ElemOf<T>;

    fn cloned_iter<'a, T: Send + Sync + Clone + 'a, I: Iterator<Item = &'a T>>(
        iter: Self::IterOf<I>,
    ) -> Self::IterOf<Cloned<I>>;

    fn copied_iter<'a, T: Send + Sync + Copy + 'a, I: Iterator<Item = &'a T>>(
        iter: Self::IterOf<I>,
    ) -> Self::IterOf<Copied<I>>;
}

pub struct Value;

impl Element for Value {
    type ElemOf<T>
        = T
    where
        T: Send + Sync;

    type IterOf<I>
        = I
    where
        I: Iterator;

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T {
        element
    }

    fn cloned_elem<T: Send + Sync + Clone>(elem: Self::ElemOf<&T>) -> Self::ElemOf<T> {
        elem.clone()
    }

    fn cloned_iter<'a, T: Send + Sync + Clone + 'a, I: Iterator<Item = &'a T>>(
        iter: Self::IterOf<I>,
    ) -> Self::IterOf<Cloned<I>> {
        iter.cloned()
    }

    fn copied_iter<'a, T: Send + Sync + Copy + 'a, I: Iterator<Item = &'a T>>(
        iter: Self::IterOf<I>,
    ) -> Self::IterOf<Copied<I>> {
        iter.copied()
    }
}

pub struct IdxValue;

impl Element for IdxValue {
    type ElemOf<T>
        = (usize, T)
    where
        T: Send + Sync;

    type IterOf<I>
        = (usize, I)
    where
        I: Iterator;

    fn item_from_element<T: Send + Sync>(element: Self::ElemOf<T>) -> T {
        element.1
    }

    fn cloned_elem<T: Send + Sync + Clone>(elem: Self::ElemOf<&T>) -> Self::ElemOf<T> {
        (elem.0, elem.1.clone())
    }

    fn cloned_iter<'a, T: Send + Sync + Clone + 'a, I: Iterator<Item = &'a T>>(
        iter: Self::IterOf<I>,
    ) -> Self::IterOf<Cloned<I>> {
        (iter.0, iter.1.cloned())
    }

    fn copied_iter<'a, T: Send + Sync + Copy + 'a, I: Iterator<Item = &'a T>>(
        iter: Self::IterOf<I>,
    ) -> Self::IterOf<Copied<I>> {
        (iter.0, iter.1.copied())
    }
}
