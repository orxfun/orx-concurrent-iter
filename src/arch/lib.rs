#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod has_more;
/// Module defining concurrent iterator traits and implementations.
pub mod iter;
mod next;

pub use has_more::HasMore;
pub use iter::buffered::buffered_chunk::{BufferedChunk, BufferedChunkX};
pub use iter::cloned::{Cloned, IntoCloned};
pub use iter::constructors::con_iterable::ConcurrentIterable;
pub use iter::constructors::into_con_iter::{IntoConcurrentIter, IterIntoConcurrentIter};
pub use iter::constructors::into_con_iter_x::IntoConcurrentIterX;
pub use iter::copied::{Copied, IntoCopied};
pub use iter::implementors::{
    iter::ConIterOfIter, iter_x::ConIterOfIterX, range::ConIterOfRange, slice::ConIterOfSlice,
    vec::ConIterOfVec,
};
pub use iter::wrappers::{ids_and_values::ConIterIdsAndValues, values::ConIterValues};
pub use iter::{con_iter::ConcurrentIter, con_iter_x::ConcurrentIterX};
pub use next::{Next, NextChunk};
