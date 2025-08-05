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

mod concurrent_collection;
mod concurrent_collection_mut;
mod concurrent_drainable;
mod concurrent_iter;
mod concurrent_iterable;
mod exact_size_concurrent_iter;
/// Implementations of concurrent iterators.
pub mod implementations;
mod into_concurrent_iter;
/// Module for creating special iterators.
pub mod iter;
mod iter_into_concurrent_iter;
mod pullers;

// exported modules: transformations

/// Cloned transformation of concurrent iterators.
pub mod cloned;
/// Copied transformation of concurrent iterators.
pub mod copied;
/// Enumerated transformation of concurrent iterators.
pub mod enumerate;

// exported types

pub use concurrent_collection::ConcurrentCollection;
pub use concurrent_collection_mut::ConcurrentCollectionMut;
pub use concurrent_drainable::ConcurrentDrainableOverSlice;
pub use concurrent_iter::ConcurrentIter;
pub use concurrent_iterable::ConcurrentIterable;
pub use exact_size_concurrent_iter::ExactSizeConcurrentIter;
pub use into_concurrent_iter::IntoConcurrentIter;
pub use iter_into_concurrent_iter::IterIntoConcurrentIter;
pub use pullers::{
    ChunkPuller, EnumeratedItemPuller, FlattenedChunkPuller, FlattenedEnumeratedChunkPuller,
    ItemPuller,
};

#[test]
fn abc() {
    use crate::*;

    fn parallel_find<T, F>(
        num_threads: usize,
        con_iter: impl ConcurrentIter<Item = T>,
        predicate: F,
    ) -> Option<T>
    where
        T: Send,
        F: Fn(&T) -> bool + Sync,
    {
        std::thread::scope(|s| {
            (0..num_threads)
                .map(|_| {
                    s.spawn(|| {
                        con_iter
                            .item_puller()
                            .find(&predicate)
                            // once found, immediately jump to end
                            .inspect(|_| con_iter.skip_to_end())
                    })
                })
                .filter_map(|x| x.join().unwrap())
                .next()
        })
    }

    let data: Vec<_> = (0..1000).map(|x| x.to_string()).collect();
    let value = parallel_find(4, data.con_iter(), |x| x.starts_with("33"));

    assert_eq!(value, Some(&33.to_string()));
}
