#![doc = include_str!("../README.md")]
#![warn(
    // missing_docs,
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

mod concurrent_collection;
mod concurrent_iter;
mod concurrent_iterable;
mod enumeration;
pub mod implementations;
mod into_concurrent_iter;
mod pullers;
pub mod wrappers;

pub use concurrent_collection::ConcurrentCollection;
pub use concurrent_iter::ConcurrentIter;
pub use concurrent_iterable::ConcurrentIterable;
pub use implementations::IterIntoConcurrentIter;
pub use into_concurrent_iter::IntoConcurrentIter;
pub use pullers::ChunkPuller;
pub use wrappers::{IntoClonedConcurrentIter, IntoCopiedConcurrentIter};
