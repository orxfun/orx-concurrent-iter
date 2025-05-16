use super::slice::RawJaggedSlice;
use crate::implementations::jagged_arrays::JaggedIndexer;

pub trait AsRawJaggedRef<'a, T, X>: Default
where
    X: JaggedIndexer,
{
    fn slice(&self, f: usize, begin_i: usize, len: usize) -> Option<&'a [T]>;

    fn jagged_slice(
        &self,
        flat_begin_idx: usize,
        flat_end_idx: usize,
    ) -> RawJaggedSlice<'a, Self, X, T>;

    fn len(&self) -> usize;

    fn len_of(&self, f: usize) -> usize;

    fn get(&self, flat_idx: usize) -> Option<&'a T>;

    // provided

    fn jagged_slice_from(&self, flat_begin_idx: usize) -> RawJaggedSlice<'a, Self, X, T> {
        self.jagged_slice(flat_begin_idx, self.len())
    }
}
