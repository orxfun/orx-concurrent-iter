use super::{con_iter::ConIterJaggedRef, raw_jagged_ref::RawJaggedRef};
use crate::{
    IntoConcurrentIter,
    implementations::jagged_arrays::{JaggedIndexer, Slices},
};

impl<'a, T, S, X> IntoConcurrentIter for RawJaggedRef<'a, T, S, X>
where
    T: Sync + 'a,
    X: JaggedIndexer,
    S: Slices<'a, T>,
{
    type Item = &'a T;

    type IntoIter = ConIterJaggedRef<'a, T, S, X>;

    fn into_con_iter(self) -> Self::IntoIter {
        ConIterJaggedRef::new(self, 0)
    }
}
