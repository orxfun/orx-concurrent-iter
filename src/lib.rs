//! # orx-concurrent-iter
//!
//! [![orx-concurrent-iter crate](https://img.shields.io/crates/v/orx-concurrent-iter.svg)](https://crates.io/crates/orx-concurrent-iter)
//! [![orx-concurrent-iter documentation](https://docs.rs/orx-concurrent-iter/badge.svg)](https://docs.rs/orx-concurrent-iter)
//!
//! A thread-safe, ergonomic and lightweight concurrent iterator trait and efficient implementations.
//!
//! * **ergonomic**: An iterator implementing `ConcurrentIter` can safely be shared among threads. It may be iterated over concurrently from multiple threads with `for` or `while let`. It further provides higher level methods such as `for_each` and `fold` which allow for safe, simple and efficient parallelism.
//! * **efficient** and **lightweight**: All concurrent iterator implementations provided in this crate extend atomic iterators which are lock-free and depend only on atomic primitives.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! A `ConcurrentIter` can be safely shared among threads and iterated over concurrently. As expected, it will yield each element only once and in order. The yielded elements will be shared among the threads which concurrently iterates based on first come first serve. In other words, threads concurrently pull remaining elements from the iterator.
//!
//! ```rust
//! use orx_concurrent_iter::*;
//! use std::fmt::Debug;
//!
//! fn fake_work<T: Debug>(_x: T) {
//!     std::thread::sleep(std::time::Duration::from_nanos(10));
//! }
//!
//! /// `process` elements of `con_iter` concurrently using `num_threads`
//! fn process_concurrently<T, I, F>(process: &F, num_threads: usize, iter: I)
//! where
//!     T: Send + Sync,
//!     F: Fn(T) + Send + Sync,
//!     I: ConcurrentIter<Item = T>,
//! {
//!     std::thread::scope(|s| {
//!         for _ in 0..num_threads {
//!             s.spawn(|| {
//!                 // concurrently iterate over values in a `for` loop
//!                 for value in iter.values() {
//!                     process(value);
//!                 }
//!             });
//!         }
//!     });
//! }
//!
//! /// executes `fake_work` concurrently on all elements of the `concurrent_iter`
//! fn run<T: Send + Sync + Debug>(concurrent_iter: impl ConcurrentIter<Item = T>) {
//!     process_concurrently(&fake_work, 8, concurrent_iter)
//! }
//!
//! // non-consuming iteration over references
//! let names: [String; 3] = [
//!     String::from("foo"),
//!     String::from("bar"),
//!     String::from("baz"),
//! ];
//! run::<&String>(names.con_iter());
//!
//! let values: Vec<i32> = (0..8).map(|x| 3 * x + 1).collect();
//! run::<&i32>(values.con_iter());
//!
//! let slice: &[i32] = values.as_slice();
//! run::<&i32>(slice.con_iter());
//!
//! // consuming iteration over values
//! run::<String>(names.into_con_iter());
//! run::<i32>(values.into_con_iter());
//!
//! // any Iterator into ConcurrentIter
//! let values: Vec<i32> = (0..1024).collect();
//!
//! let iter_ref = values.iter().filter(|x| *x % 2 == 0);
//! run::<&i32>(iter_ref.into_con_iter());
//!
//! let iter_val = values
//!     .iter()
//!     .filter(|x| *x % 2 == 0)
//!     .map(|x| (7 * x + 3) as usize)
//!     .skip(2)
//!     .take(5);
//! run::<usize>(iter_val.into_con_iter());
//! ```
//!
//! ### Ways to Loop
//!
//! `ConcurrentIter`s implement the `next` method, which is a concurrent counterpart of `Iterator::next`. Therefore, the iterator can be used almost the same as a regular `Iterator` safely across multiple threads. Slight difference of different ways to iterate over a `ConcurrentIter` are demonstrated and explained in the following example.
//!
//! ```rust
//! use orx_concurrent_iter::*;
//! use std::fmt::Debug;
//!
//! fn process_concurrently<T, I, F>(process: &F, num_threads: usize, iter: I)
//! where
//!     T: Send + Sync + Debug,
//!     F: Fn(T) + Send + Sync,
//!     I: ConcurrentIter<Item = T>,
//! {
//!     std::thread::scope(|s| {
//!         for _ in 0..num_threads {
//!             s.spawn(|| {
//!                 // a.1 -> pull elements 1-by-1
//!                 for value in iter.values() {
//!                     dbg!(&value);
//!                 }
//!
//!                 while let Some(value) = iter.next() {
//!                     dbg!(&value);
//!                 }
//!
//!                 // a.2 -> pull elements and their indices 1-by-1
//!                 for (idx, value) in iter.ids_and_values() {
//!                     dbg!(idx, value);
//!                 }
//!
//!                 while let Some(x) = iter.next_id_and_value() {
//!                     dbg!(x.idx, x.value);
//!                 }
//!
//!                 // b.1 -> pull elements 16-by-16
//!                 let mut chunk_iter = iter.buffered_iter(16);
//!                 while let Some(chunk) = chunk_iter.next() {
//!                     for value in chunk.values {
//!                         dbg!(value);
//!                     }
//!                 }
//!
//!                 // b.2 -> pull elements and their indices 16-by-16
//!                 let mut chunk_iter = iter.buffered_iter(16);
//!                 while let Some(chunk) = chunk_iter.next() {
//!                     for (i, value) in chunk.values.enumerate() {
//!                         let idx = chunk.begin_idx + i;
//!                         dbg!(idx, value);
//!                     }
//!                 }
//!             });
//!         }
//!     });
//! }
//! ```
//!
//! * `for` and `while let` loops of **a.1** demonstrate the most basic usage where threads will pull the next element of the iterator whenever they complete processing the prior element.
//! * The loops in **a.2** can be considered as the `enumerate`d counterpart. Note that each thread will pull different elements at different positions of the iterator depending on how fast they finish the execution inside the loop. Therefore, an `enumerate` call inside the thread, or counting the pulled elements by that particular thread, does not provide the index of the element in the original data source. `ConcurrentIter` additionally provides the original index with `ids_and_values` or `next_id_and_value` methods.
//! * Whenever the work to be done inside the loop is too small (like just the `dbg` call in the above example), taking elements 1-by-1 might be suboptimal. In such cases, a better idea is to pull elements in batches. In **b.1**, we create a buffered chunk iterator which pulls 16 (or less, if not enough left) **consecutive** elements at each `next` call. Note that `chunk` returned by `chunk_iter.next()` is an `ExactSizeIterator` with a known `len`.
//! * Similar to before, **b.2** is the counterpart of **b.1** which allows us to use the original `idx` of the elements. `chunk.begin_idx` represents the original index of the first element of the returned `chunk.values` iterator. Note that `chunk.values` is always non-empty; i.e., always has at least one element, otherwise, `next` returns `None`. Further, the chunk iterator contains consecutive elements. Hence, we can get the original indices of all elements by combining `chunk.begin_idx` with the local indices of the current `chunk` obtained by the `chunk.values.enumerate`; i.e., `let idx = chunk.begin_idx + i`.
//!
//!
//! ### Parallel Fold
//!
//! Considering the elements of the iteration as inputs of a process, `ConcurrentIter` conveniently allows distribution of tasks to multiple threads. See below a parallel fold implementation using the concurrent iterator.
//!
//! ```rust
//! use orx_concurrent_iter::*;
//!
//! fn compute(input: u64) -> u64 {
//!     input * 2
//! }
//!
//! fn fold(aggregated: u64, value: u64) -> u64 {
//!     aggregated + value
//! }
//!
//! fn par_fold(num_threads: usize, inputs: impl ConcurrentIter<Item = u64>) -> u64 {
//!     std::thread::scope(|s| {
//!         (0..num_threads)
//!             .map(|_| s.spawn(|| inputs.values().map(compute).fold(0u64, fold)))
//!             .collect::<Vec<_>>()
//!             .into_iter()
//!             .map(|x| x.join().expect("-_-"))
//!             .fold(0u64, fold)
//!     })
//! }
//!
//! // validate
//! for num_threads in [1, 2, 4, 8] {
//!     let values = (0..1024).map(|x| 2 * x);
//!     let par_result = par_fold(num_threads, values.into_con_iter());
//!     assert_eq!(par_result, 2 * 1023 * 1024);
//! }
//! ```
//!
//! Notes on the implementation:
//! * The entire parallel fold implementation is 7 lines.
//! * Parallel fold operation is defined as two fold operations. This is straightforward and easy to reason about.
//!   * The first `.map(_).fold(_)` defines the parallel fold operation executed by `num_threads` threads. Each thread returns its own aggregated result.
//!   * The second `map(_).fold(_)` defines the final sequential fold operation executed to fold over the `num_threads` results obtained by each thread.
//!
//! ### Parallel Map
//!
//! Parallel map can also be implemented by merging returned transformed collections, such as vectors. Especially for larger data types, a more efficient approach could be to pair `ConcurrentIter` with a concurrent collection such as [`orx_concurrent_bag::ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) which allows to efficiently collect results concurrently without copies.
//!
//! ```rust
//! use orx_concurrent_iter::*;
//! use orx_concurrent_bag::*;
//!
//! fn map(input: u64) -> String {
//!     input.to_string()
//! }
//!
//! fn parallel_map(num_threads: usize, iter: impl ConcurrentIter<Item = u64>) -> SplitVec<String> {
//!     let outputs = ConcurrentBag::new();
//!     std::thread::scope(|s| {
//!         for _ in 0..num_threads {
//!             s.spawn(|| {
//!                 for output in iter.values().map(map) {
//!                     outputs.push(output);
//!                 }
//!             });
//!         }
//!     });
//!     outputs.into_inner()
//! }
//!
//! // test
//! for num_threads in [1, 2, 4, 8] {
//!     let inputs = (0..1024).map(|x| 2 * x);
//!     let outputs = parallel_map(num_threads, inputs.into_con_iter());
//!     assert_eq!(1024, outputs.len());
//! }
//! ```
//!
//! Note that due to the way how the concurrent iterator works, this parallel map implementation is suitable for heterogeneous inputs without requiring much care, as illustrated below:
//! * Assume that our input `iter` contains 10 elements which can all be mapped homogeneously in exactly 1s, except for the first element.
//! * The first element takes 5 seconds to map; i.e., to process.
//! * Further assume that we have 2 threads available, threads `A` and `B`, and thread `A` will pick the first element. Then, the parallel execution will likely happen in the following manner:
//!   * `A` works on mapping element 0 in intervals [t1, t5].
//!   * Meanwhile, `B` pulls and maps elements 1, 2, ..., 5 in intervals [t1, t5].
//!   * In interval [t6, t7], both threads will pull and map elements 6, 7, 8, 9 in arbitrary order.
//! * Since the threads pull elements as soon as they are idle, this implementation leads to an efficient parallelization strategy without any assumption or knowledge about the duration of the individual tasks.
//!
//! Note that due to parallelization, `outputs` is not guaranteed to be in the same order as `inputs`. In order to preserve the order of the input in the output, iteration with indices can be used to sort the result accordingly. Alternative to post-sorting, `ConcurrentBag` can be replaced with [`orx_concurrent_bag::ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag) to already collect in order.
//!
//! ### Parallel Find, A Little Communication Among Threads
//!
//! As illustrated above, efficient parallel implementations of many methods are conveniently possible with `ConcurrentIter`. There is only one bit of information implicitly shared among threads by the concurrent iterator: the elements left. In scenarios where we do not need to iterate over all elements, we can use, actually change, this information to share a message among threads. We might call such cases as **early-return** scenarios. An obvious example is the `find` method, where we are looking for an element and we want to terminate the search as soon as we find a match.
//!
//! You may see a parallel implementation of the find method below.
//!
//! ```rust
//! use orx_concurrent_iter::*;
//!
//! fn par_find<I, P>(iter: I, predicate: P, n_threads: usize) -> Option<(usize, I::Item)>
//! where
//!     I: ConcurrentIter,
//!     P: Fn(&I::Item) -> bool + Send + Sync,
//! {
//!     std::thread::scope(|s| {
//!         (0..n_threads)
//!             .map(|_| {
//!                 s.spawn(|| {
//!                     for (i, x) in iter.ids_and_values() {
//!                         if predicate(&x) {
//!                             iter.skip_to_end();
//!                             return Some((i, x));
//!                         }
//!                     }
//!                     None
//!                 })
//!             })
//!             .collect::<Vec<_>>()
//!             .into_iter()
//!             .flat_map(|x| x.join().expect("(-)"))
//!             .min_by_key(|x| x.0)
//!     })
//! }
//!
//! let mut names: Vec<_> = (0..8785).map(|x| x.to_string()).collect();
//! names[42] = "foo".to_string();
//!
//! let result = par_find(names.con_iter(), |x| x.starts_with('x'), 4);
//! assert_eq!(result, None);
//!
//! let result = par_find(names.con_iter(), |x| x.starts_with('f'), 4);
//! assert_eq!(result, Some((42, &"foo".to_string())));
//!
//! names[43] = "foo_second_match".to_string();
//! let result = par_find(names.con_iter(), |x| x.starts_with('f'), 4);
//! assert_eq!(result, Some((42, &"foo".to_string())));
//! ```
//!
//! Notice that the parallel find implementation is in two folds:
//! * (parallel search) Inside each thread, we loop through the elements of the concurrent iterator and return the first value satisfying the `predicate` together with its index.
//! * (sequential wrap up) Since this is a parallel execution, we might end up receiving multiple matches from multiple threads. In the second part, we investigate the thread results and return the one with the minimum position index (`min_by_key(|x| x.0)`) since that is the element which appears first in the original iterator.
//!
//! So far, this is straightforward and similar to the parallel fold implementation. The difference; however, is the additional `iter.skip_to_end()` call. This call will immediately consume all remaining elements of the iterator. Therefore, whenever, another thread tries to advance the iterator in the `for (i, x) in iter.ids_and_values()`, it will not receive any further elements. Hence, they will as well return as soon as they complete processing their last pulled element. This establishes a very trivial communication among threads, which is critical in achieving efficiency in early return scenarios, as the find method. To demonstrate, assume the case we didn't use `iter.skip_to_end()` in the above implementation.
//! * In the second example, the iterator has 8785 elements where there exists only one element satisfying the predicate, "foo" at position 42.
//! * One of the 4 threads used, say `A`, will find this element and return immediately.
//! * The other 3 threads will never see this element, since it is pulled by `A`. They will iterate over all remaining elements and will eventually return `None`.
//! * The final result will be correct. However, this implementation will evaluate all elements of the iterator regardless of where the first matching element is. Although parallelized, this would be a very inefficient implementation.
//!
//!
//! ## Traits and Implementors
//!
//! As discussed so far, the trait defining types that can be safely be iterated concurrently by multiple threads is `ConcurrentIter`.
//!
//! Further, there are two traits which define types that can provide a `ConcurrentIter`.
//! * A `ConcurrentIterable` type implements the **`con_iter(&self)`** method which returns a concurrent iterator without consuming the type itself.
//! * On the other hand, types implementing `IntoConcurrentIter` trait has the **`into_con_iter(self)`** method which consumes and converts the type into a concurrent iterator. Additionally there exists `IterIntoConcurrentIter` trait which is functionally identical to `IntoConcurrentIter` and only implemented by regular iterators, separated only to allow for special implementations for vectors and arrays.
//!
//! The following table summarizes the implementations of the standard types in this crate.
//!
//! | Type | ConcurrentIterable <br/> `con_iter()` element type | IntoConcurrentIter <br/> `into_con_iter()` element type | Cloned <br/> `con_iter().cloned()` |
//! |---|---|---|---|
//! | `&'a [T]` | `&'a T` | `&'a T` | `T` |
//! | `Range<Idx>` | `Idx` | `Idx` | |
//! | `Vec<T>` | `&T` | `T` | |
//! | `[T; N]` | `&T` | `T` | |
//! | `Iter: Iterator<Item = T>` | - | `T` | |
//!
//!
//! ## License
//!
//! This library is licensed under MIT license. See LICENSE for details.

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

mod has_more;
/// Module defining concurrent iterator traits and implementations.
pub mod iter;
mod next;

pub use has_more::HasMore;
pub use iter::atomic_counter::AtomicCounter;
pub use iter::cloned::{Cloned, IntoCloned};
pub use iter::con_iter::{ConcurrentIter, ExactSizeConcurrentIter};
pub use iter::constructors::con_iterable::ConcurrentIterable;
pub use iter::constructors::into_con_iter::{IntoConcurrentIter, IterIntoConcurrentIter};
pub use iter::constructors::into_exact_con_iter::IntoExactSizeConcurrentIter;
pub use iter::implementors::{
    array::ConIterOfArray, iter::ConIterOfIter, range::ConIterOfRange, slice::ConIterOfSlice,
    vec::ConIterOfVec,
};
pub use iter::wrappers::{ids_and_values::ConIterIdsAndValues, values::ConIterValues};
pub use next::{Next, NextChunk};

mod tests {
    use crate::*;

    #[test]
    fn abc() {
        let vec = vec![0, 1, 2, 3, 4];
        let source = vec.iter().map(|x| 2 * x);
        assert_eq!(source.size_hint(), (5, Some(5)));
        // assert_eq!(source.try_get_exact_len(), Some(5));
        let iter = source.into_con_iter();

        // assert_eq!(iter.try_get_len(), Some(5));
    }
}
