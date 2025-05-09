pub struct JaggedIndex {
    f: usize,
    i: usize,
}

impl JaggedIndex {
    pub fn new(f: usize, i: usize) -> Self {
        Self { f, i }
    }
}
