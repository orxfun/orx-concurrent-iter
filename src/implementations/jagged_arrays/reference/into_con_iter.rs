use super::{con_iter::ConIterJaggedRef, raw_jagged_ref::RawJaggedRef};
use crate::{
    IntoConcurrentIter,
    implementations::jagged_arrays::{JaggedIndexer, as_slice::AsSlice},
};

impl<'a, T, S, X> IntoConcurrentIter for RawJaggedRef<'a, T, S, X>
where
    T: Send + Sync + 'a,
    X: JaggedIndexer,
    S: AsSlice<T> + Send + Sync,
{
    type Item = &'a T;

    type IntoIter = ConIterJaggedRef<'a, T, S, X>;

    fn into_con_iter(self) -> Self::IntoIter {
        ConIterJaggedRef::new(self, 0)
    }
}
