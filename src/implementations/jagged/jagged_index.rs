#[derive(PartialEq, Debug)]
pub struct JaggedIndex {
    pub f: usize,
    pub i: usize,
}

impl JaggedIndex {
    pub fn new(f: usize, i: usize) -> Self {
        Self { f, i }
    }
}
