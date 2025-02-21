// use super::mut_handle::{AtomicState, MutHandle, AVAILABLE, COMPLETED};
// use crate::{
//     concurrent_iter::ConcurrentIter,
//     enumeration::{Element, Enumeration, Regular},
// };
// use core::{cell::UnsafeCell, sync::atomic::Ordering};

// pub struct ConIterXOfIter<I, T>
// where
//     T: Send + Sync,
//     I: Iterator<Item = T>,
// {
//     pub(super) iter: UnsafeCell<I>,
//     initial_len: Option<usize>,
//     state: AtomicState,
// }
