use super::raw_jagged_slice::RawJaggedSlice;

pub struct RawJaggedSliceIterRef<'a, T> {
    slice: RawJaggedSlice<'a, T>,
    f: usize,
    current: core::slice::Iter<'a, T>,
}

impl<'a, T> RawJaggedSliceIterRef<'a, T> {
    pub(super) fn new(slice: RawJaggedSlice<'a, T>) -> Self {
        let f = 0;
        let first_slice = slice.get_slice(f).unwrap_or(Default::default());
        let current = first_slice.iter();
        Self { slice, f, current }
    }

    fn next_slice(&mut self) -> Option<&'a T> {
        match self.f == self.slice.num_slices() - 1 {
            false => {
                self.f += 1;
                let new_slice = self.slice.get_slice(self.f).unwrap_or(Default::default());
                self.current = new_slice.iter();
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
