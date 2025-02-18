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

mod chunk_puller;
mod chunks_iter;
mod concurrent_collection;
mod concurrent_iter;
mod concurrent_iterable;
mod enumeration;
mod into_concurrent_iter;

mod implementations;

pub use concurrent_iterable::ConcurrentIterable;
pub use into_concurrent_iter::IntoConcurrentIter;
