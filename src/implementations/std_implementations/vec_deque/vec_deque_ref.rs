use crate::implementations::jagged_arrays::Slices;
use std::{collections::VecDeque, vec};

pub struct VecDequeRef<'a, T>(Option<&'a VecDeque<T>>);

impl<'a, T> VecDequeRef<'a, T> {
    pub(super) fn new(vec_deque_ref: &'a VecDeque<T>) -> Self {
        Self(Some(vec_deque_ref))
    }
}

impl<'a, T: 'a> Clone for VecDequeRef<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a, T: 'a> Slices<'a, T> for VecDequeRef<'a, T> {
    fn empty() -> Self {
        Self(None)
    }

    fn num_slices(&self) -> usize {
        self.0.map(|x| x.len()).unwrap_or(0)
    }

    fn slices(&self) -> impl Iterator<Item = &'a [T]> {
        self.0
            .map(|x| {
                let (a, b) = x.as_slices();
                vec![a, b].into_iter()
            })
            .unwrap_or_default()
    }

    fn lengths(&self) -> impl Iterator<Item = usize> {
        match &self.0 {
            Some(x) => {
                let (a, b) = x.as_slices();
                vec![a.len(), b.len()].into_iter()
            }
            None => Default::default(),
        }
    }

    fn slice_at(&self, f: usize) -> Option<&'a [T]> {
        self.0.and_then(|x| match f {
            0 => Some(x.as_slices().0),
            1 => Some(x.as_slices().1),
            _ => None,
        })
    }

    unsafe fn slice_at_unchecked(&self, f: usize) -> &'a [T] {
        let x = self.0.expect("`slice_at_unchecked` called when empty");
        match f {
            0 => x.as_slices().0,
            _ => x.as_slices().1,
        }
    }
}
