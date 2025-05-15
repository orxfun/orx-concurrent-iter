pub trait RawJaggedStorage: Default {
    type Item;

    /// Number of arrays in the storage.
    fn num_arrays(&self) -> usize;

    /// Returns an iterator of lengths of individual arrays forming the jagged array.
    fn arr_lengths(&self) -> impl Iterator<Item = usize>;

    /// Returns the length of the `f`-th array.
    ///
    /// # Panics
    ///
    /// Panics if `f >= num_arrays()`.
    fn arr_length_of(&self, f: usize) -> usize;

    /// Returns pointers to the first and last, (len-1)-th, element of the `f`-th array of the jagged array.
    ///
    /// If the corresponding array is empty, both pointers are null.
    ///
    /// # Panics
    ///
    /// Panics if `f` is out of bounds.
    fn first_and_last_ptrs_of(&self, f: usize) -> [*const Self::Item; 2];
}
