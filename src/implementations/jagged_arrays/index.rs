use super::AsRawSlice;
use core::cmp::Ordering;

#[derive(Default, PartialEq, Debug, Clone)]
pub struct JaggedIndex {
    pub f: usize,
    pub i: usize,
}

impl From<(usize, usize)> for JaggedIndex {
    fn from((f, i): (usize, usize)) -> Self {
        Self::new(f, i)
    }
}

impl From<[usize; 2]> for JaggedIndex {
    fn from([f, i]: [usize; 2]) -> Self {
        Self::new(f, i)
    }
}

impl JaggedIndex {
    pub fn new(f: usize, i: usize) -> Self {
        Self { f, i }
    }

    pub fn is_in_exc_bounds_of<T>(&self, slices: &[impl AsRawSlice<T>]) -> bool {
        match slices.is_empty() {
            true => self.f == 0 && self.i == 0,
            false => self.f < slices.len() && self.i <= slices[self.f].length(),
        }
    }

    pub fn is_in_inc_bounds_of<T>(&self, slices: &[impl AsRawSlice<T>]) -> bool {
        match slices.is_empty() {
            true => self.f == 0 && self.i == 0,
            false => self.f < slices.len() && self.i < slices[self.f].length(),
        }
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
