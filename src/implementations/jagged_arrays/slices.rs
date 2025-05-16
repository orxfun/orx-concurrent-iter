use alloc::vec::Vec;

/// A collection of slices.
pub trait Slices<'a, T>: Clone
where
    T: 'a,
{
    /// Creates an empty collection of slices.
    fn empty() -> Self;

    /// Number of slices.
    fn num_slices(&self) -> usize;

    /// Iterator over slices.
    fn slices(&self) -> impl Iterator<Item = &'a [T]>;

    /// Iterator over lengths of slices.
    fn lengths(&self) -> impl Iterator<Item = usize>;

    /// Returns a reference to the `f`-th slice.
    /// Returns None if out of bounds.
    fn slice_at(&self, f: usize) -> Option<&'a [T]>;

    /// Returns the `f`-th slice without bounds checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `f < self.num_slices()`.
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
