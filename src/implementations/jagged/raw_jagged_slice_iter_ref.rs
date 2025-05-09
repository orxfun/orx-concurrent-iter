use super::{raw_jagged_slice::RawJaggedSlice, raw_slice::RawSlice};

pub struct RawJaggedSliceIterRef<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, T> RawJaggedSliceIterRef<'a, T> {
    pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        let f = 0;
        let current = slice.get_slice(f).unwrap_or(Default::default());
        Self { slice, f, current }
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        match self.f == self.slice.num_slices() - 1 {
            false => {
                self.f += 1;
                self.current = slice.get_slice(f).unwrap_or(Default::default());
                self.next()
            }
            true => None,
        }
    }
}

impl<'a, T> Iterator for RawJaggedSliceIterRef<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next();
        match next.is_some() {
            true => next,
            false => self.next_slice(),
        }
    }
}
