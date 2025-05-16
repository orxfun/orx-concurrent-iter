use super::AsRawSlice;
use core::cmp::Ordering;

/// Index of an element in a jagged array.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct JaggedIndex {
    /// Index of the array containing the element.
    pub f: usize,
    /// Index of the element within the array containing it.
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
    /// Creates a new jagged index:
    ///
    /// * `f`: index of the array containing the element.
    /// * `i`: index of the element within the array containing it.
    pub fn new(f: usize, i: usize) -> Self {
        Self { f, i }
    }

    pub(super) fn is_in_exc_bounds_of<T>(&self, slices: &[impl AsRawSlice<T>]) -> bool {
        match slices.is_empty() {
            true => self.f == 0 && self.i == 0,
            false => self.f < slices.len() && self.i <= slices[self.f].length(),
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
