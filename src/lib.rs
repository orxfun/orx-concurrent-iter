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
mod exact_size_concurrent_iter;
mod implementations;
mod into_concurrent_iter;
mod iter_into_concurrent_iter;
mod pullers;

// exported modules: transformations

pub mod cloned;
pub mod copied;
pub mod enumerate;

// exported types

pub use concurrent_iter::ConcurrentIter;
pub use into_concurrent_iter::IntoConcurrentIter;
pub use iter_into_concurrent_iter::IterIntoConcurrentIter;
pub use pullers::ChunkPuller;
