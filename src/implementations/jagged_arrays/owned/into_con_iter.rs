use super::{ConIterJaggedOwned, RawJagged};
use crate::{IntoConcurrentIter, implementations::jagged_arrays::JaggedIndexer};

impl<T, X> IntoConcurrentIter for RawJagged<T, X>
where
    X: JaggedIndexer,
{
    type Item = T;

    type IntoIter = ConIterJaggedOwned<T, X>;

    fn into_con_iter(self) -> Self::IntoIter {
        ConIterJaggedOwned::new(self, 0)
    }
}
