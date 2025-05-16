use alloc::vec::Vec;

pub trait Slices<'a, T>: Clone
where
    T: 'a,
{
    fn empty() -> Self;

    fn num_slices(&self) -> usize;

    fn slices(&self) -> impl Iterator<Item = &'a [T]>;

    fn lengths(&self) -> impl Iterator<Item = usize>;

    fn slice_at(&self, f: usize) -> Option<&'a [T]>;

    unsafe fn slice_at_unchecked(&self, f: usize) -> &'a [T];
}

impl<'a, T: 'a> Slices<'a, T> for &'a [Vec<T>] {
    fn empty() -> Self {
        Default::default()
    }

    fn num_slices(&self) -> usize {
        self.len()
    }

    fn slices(&self) -> impl Iterator<Item = &'a [T]> {
        self.iter().map(|x| x.as_slice())
    }

    fn lengths(&self) -> impl Iterator<Item = usize> {
        self.iter().map(|x| x.len())
    }

    fn slice_at(&self, f: usize) -> Option<&'a [T]> {
        self.get(f).map(|x| x.as_slice())
    }

    unsafe fn slice_at_unchecked(&self, f: usize) -> &'a [T] {
        self[f].as_slice()
    }
}
