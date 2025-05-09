use super::raw_jagged_slice::RawJaggedSlice;

pub struct RawJaggedSliceIterRef<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, T> Default for RawJaggedSliceIterRef<'a, T> {
    fn default() -> Self {
        Self {
            slice: Default::default(),
            f: Default::default(),
            current: Default::default(),
        }
    }
}

impl<'a, T> RawJaggedSliceIterRef<'a, T> {
    pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        Self {
            slice,
            ..Default::default()
        }
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        match self.f == self.slice.num_slices() {
            false => {
                let slice = self.slice.get_slice(self.f).unwrap_or(Default::default());
                self.current = slice.iter();
                self.f += 1;
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
