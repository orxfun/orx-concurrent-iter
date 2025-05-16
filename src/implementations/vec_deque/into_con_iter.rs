use crate::{IntoConcurrentIter, implementations::jagged_arrays::ConIterJaggedRef};
use std::collections::VecDeque;

// impl<'a, T> IntoConcurrentIter for &'a VecDeque<T>
// where
//     T: Send + Sync,
// {
//     type Item = &'a T;

//     type IntoIter = ConIterJaggedRef<'a, T, Fragment<T>, G>;

//     fn into_con_iter(self) -> Self::IntoIter {
//         Self::IntoIter::new(self)
//     }
// }

fn abc(vec: VecDeque<usize>) {
    let x = vec.as_slices();
    //
}
