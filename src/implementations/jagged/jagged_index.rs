use super::raw_slice::RawSlice;
use core::cmp::Ordering;

#[derive(Default, PartialEq, Debug)]
pub struct JaggedIndex {
    pub f: usize,
    pub i: usize,
}

impl JaggedIndex {
    pub fn new(f: usize, i: usize) -> Self {
        Self { f, i }
    }

    pub fn is_in_exc_bounds_of<T>(&self, slices: &[RawSlice<T>]) -> bool {
        match self.f.cmp(&slices.len()) {
            Ordering::Less => self.i <= slices[self.f].len(),
            Ordering::Equal => self.i == 0,
            Ordering::Greater => false,
        }
    }

    pub fn is_in_inc_bounds_of<T>(&self, slices: &[RawSlice<T>]) -> bool {
        self.f < slices.len() && self.i < slices[self.f].len()
    }
}

impl PartialOrd for JaggedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.f.partial_cmp(&other.f) {
            Some(Ordering::Equal) => self.i.partial_cmp(&other.i),
            ord => ord,
        }
    }
}
