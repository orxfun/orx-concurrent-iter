use crate::implementations::jagged_arrays::{
    as_raw_slice::{AsOwningSlice, AsRawSlice},
    as_slice::AsSlice,
    index::JaggedIndex,
    indexer::JaggedIndexer,
};
use orx_pseudo_default::PseudoDefault;
use std::{cmp::Ordering, marker::PhantomData};

/// Raw representation of a reference to a jagged array.
///
/// Further, jagged has an indexer which maps a flat-element-index to a
/// two-dimensional index where the first is the index of the array and
/// the second is the position of the element within this array.
pub struct RawJaggedRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    arrays: &'a [S],
    len: usize,
    indexer: X,
    phantom: PhantomData<T>,
}

impl<'a, T, S, X> Default for RawJaggedRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    fn default() -> Self {
        Self {
            arrays: Default::default(),
            len: Default::default(),
            indexer: PseudoDefault::pseudo_default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, T, S, X> RawJaggedRef<'a, T, S, X>
where
    X: JaggedIndexer,
    S: AsSlice<T>,
{
    pub(super) fn len_of(&self, f: usize) -> Option<usize> {
        self.arrays.get(f).map(|x| x.as_slice().len())
    }

    pub(super) fn slice(&self, f: usize, begin_within_slice: usize, len: usize) -> Option<&'a [T]> {
        self.arrays.get(f).and_then(|array| {
            let array = array.as_slice();
            (begin_within_slice < array.len()).then(|| {
                let ptr = unsafe { array.as_ptr().add(begin_within_slice) };
                unsafe { core::slice::from_raw_parts(ptr, len) }
            })
        })
    }
}
