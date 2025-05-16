/// A type that can be represented as a slice.
pub trait AsSlice<T> {
    /// Beginning of the slice.
    fn ptr(&self) -> *const T;

    /// Length of the slice.
    fn len(&self) -> usize;
}
