use super::slice::RawJaggedSlice;
use crate::implementations::jagged_arrays::JaggedIndexer;

pub trait AsRawJaggedRef<'a, T, X>: Default
where
    X: JaggedIndexer,
{
    fn slice(&self, f: usize, begin_within_slice: usize, len: usize) -> Option<&'a [T]>;

    fn jagged_slice(
        &self,
        flat_begin_idx: usize,
        flat_end_idx: usize,
    ) -> RawJaggedSlice<'a, Self, X, T>;

    fn len(&self) -> usize;

    fn len_of(&self, f: usize) -> usize;

    fn element(&self, flat_idx: usize) -> Option<&'a T>;

    // provided

    fn jagged_slice_from(&self, flat_begin_idx: usize) -> RawJaggedSlice<'a, Self, X, T> {
        self.jagged_slice(flat_begin_idx, self.len())
    }
}

impl<'a, T, X> AsRawJaggedRef<'a, T, X> for Vec<Vec<T>>
where
    X: JaggedIndexer,
{
    fn slice(&self, f: usize, begin_within_slice: usize, len: usize) -> Option<&'a [T]> {
        self.get(f).and_then(|array| {
            (begin_within_slice < array.len()).then(|| {
                let ptr = unsafe { array.as_ptr().add(begin_within_slice) };
                unsafe { core::slice::from_raw_parts(ptr, len) }
            })
        })
    }

    fn jagged_slice(
        &self,
        flat_begin_idx: usize,
        flat_end_idx: usize,
    ) -> RawJaggedSlice<'a, Self, X, T> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn len_of(&self, f: usize) -> usize {
        todo!()
    }

    fn element(&self, flat_idx: usize) -> Option<&'a T> {
        todo!()
    }
}
