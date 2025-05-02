use crate::{
    concurrent_iter::ConcurrentIter, exact_size_concurrent_iter::ExactSizeConcurrentIter,
    implementations::ptr_utils::take,
};
use alloc::vec::Vec;
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Concurrent iterator of a [`Vec`].
///
/// It can be created by calling [`into_con_iter`] on a vector.
///
/// [`into_con_iter`]: crate::IntoConcurrentIter::into_con_iter
///
/// # Examples
///
/// ```
/// use orx_concurrent_iter::*;
///
/// let vec = vec![1, 2];
/// let con_iter = vec.into_con_iter();
/// assert_eq!(con_iter.next(), Some(1));
/// assert_eq!(con_iter.next(), Some(2));
/// assert_eq!(con_iter.next(), None);
/// ```
pub struct ConIterHashSet<T>
where
    T: Send + Sync,
{
    ptr: *const T,
    vec_len: usize,
    vec_cap: usize,
    counter: AtomicUsize,
}

fn abc() {
    let set = BTr
    //
}
