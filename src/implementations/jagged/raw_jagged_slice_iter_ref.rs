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

    fn get_next_slice(slice: &RawJaggedSlice<'a, T>, f: usize) -> core::slice::Iter<'a, T> {
        let first_slice = slice.get_slice(f).unwrap_or(Default::default());
        first_slice.iter()
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        match self.f == self.slice.num_slices() - 1 {
            false => {
                self.current = Self::get_next_slice(&self.slice, self.f);
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
