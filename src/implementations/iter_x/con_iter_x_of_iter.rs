use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU8, AtomicUsize},
};

type State = u8;
const AVAILABLE: State = 0;
const IS_MUTATING: State = 1;
const COMPLETED: State = 2;

pub struct ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T>,
{
    iter: UnsafeCell<I>,
    initial_len: Option<usize>,
    counter: AtomicUsize,
    is_mutating: AtomicU8,
}

// TODO: drop when Vec.into_iter() for instance

impl<I, T> Default for ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T> + Default,
{
    fn default() -> Self {
        Self::new(I::default())
    }
}

impl<I, T> ConIterXOfIter<I, T>
where
    T: Send + Sync,
    I: Iterator<Item = T> + Default,
{
    fn new(iter: I) -> Self {
        let initial_len = match iter.size_hint() {
            (_, None) => None,
            (lower, Some(upper)) if lower == upper => Some(lower),
            _ => None,
        };

        Self {
            iter: iter.into(),
            initial_len,
            counter: 0.into(),
            is_mutating: AVAILABLE.into(),
        }
    }
}
