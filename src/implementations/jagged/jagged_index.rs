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
}

impl PartialOrd for JaggedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.f.partial_cmp(&other.f) {
            Some(Ordering::Equal) => self.i.partial_cmp(&other.i),
            ord => ord,
        }
    }
}
