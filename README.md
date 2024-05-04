# orx-concurrent-iter

[![orx-concurrent-iter crate](https://img.shields.io/crates/v/orx-concurrent-iter.svg)](https://crates.io/crates/orx-concurrent-iter)
[![orx-concurrent-iter documentation](https://docs.rs/orx-concurrent-iter/badge.svg)](https://docs.rs/orx-concurrent-iter)

A thread-safe, ergonomic and lightweight concurrent iterator trait and efficient implementations.

* **ergonomic**: An iterator implementing `ConcurrentIter` can safely be shared among threads. It may be iterated over concurrently from multiple threads with `for` or `while let`. It further provides higher level methods such as `for_each` and `fold` which allow for safe, simple and efficient parallelism.
* **efficient** and **lightweight**: All concurrent iterator implementations provided in this crate extend atomic iterators which are lock-free and depend only on atomic primitives.

## Examples

### Basic Usage

A `ConcurrentIter` can be safely shared among threads and iterated over concurrently. As expected, it will yield each element only once and in order. The yielded elements will be shared among the threads which concurrently iterates based on first come first serve. In other words, threads concurrently pull remaining elements from the iterator.

```rust
use orx_concurrent_iter::*;
use std::fmt::Debug;

fn fake_work<T: Debug>(_x: T) {
    std::thread::sleep(std::time::Duration::from_nanos(10));
}

/// `process` elements of `con_iter` concurrently using `num_threads`
fn process_concurrently<T, ConIter, Fun>(
    process: &Fun,
    num_threads: usize,
    concurrent_iter: ConIter,
) where
    T: Send + Sync,
    Fun: Fn(T) + Send + Sync,
    ConIter: ConcurrentIter<Item = T>,
{
    // just take a reference and share among threads
    let iter = &concurrent_iter;

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                // concurrently iterate over values in a `for` loop
                for value in iter.values() {
                    process(value);
                }
            });
        }
    });
}

/// just fix process and num_threads for brevity
fn con_run<T: Send + Sync + Debug>(concurrent_iter: impl ConcurrentIter<Item = T>) {
    process_concurrently(&fake_work, 8, concurrent_iter)
}

// non-consuming iteration over references
let names: [String; 3] = [
    String::from("foo"),
    String::from("bar"),
    String::from("baz"),
];
con_run::<&String>(names.con_iter());

let values: Vec<i32> = (0..8).map(|x| 3 * x + 1).collect();
con_run::<&i32>(values.con_iter());

let slice: &[i32] = values.as_slice();
con_run::<&i32>(slice.con_iter());

// consuming iteration over values
con_run::<String>(names.into_con_iter());
con_run::<i32>(values.into_con_iter());

// any Iterator into ConcurrentIter
let values: Vec<i32> = (0..1024).collect();

let iter_ref = values.iter().filter(|x| *x % 2 == 0);
con_run::<&i32>(iter_ref.into_con_iter());

let iter_val = values
    .iter()
    .filter(|x| *x % 2 == 0)
    .map(|x| (7 * x + 3) as usize)
    .skip(2)
    .take(5);

con_run::<usize>(iter_val.into_con_iter());
```

[`ConcurrentIter::next`] method is the concurrent counterpart of `Iterator::next` method, which can be called by a shared reference. Note that regular `Iterator`s cannot be consumed by multiple threads due to `&mut self` requirement, and hence, `ConcurrentIter` does not implement `Iterator`. However, it can be used in a similar way as follows:

```rust ignore
while let Some(value) = con_iter.next() {
    process(value);
}
```

Or even with `for` loop using the `values` method:

```rust ignore
for value in con_iter.values() {
    process(value);
}
```

### Parallel Computing, Almost Trivial

Considering the elements of the iteration as inputs of a process, `ConcurrentIter` conveniently allows distribution of tasks to multiple threads.

```rust
use orx_concurrent_iter::*;

fn compute(input: u64) -> u64 {
    std::thread::sleep(std::time::Duration::from_nanos(2));
    input
}

fn fold(aggregated: u64, value: u64) -> u64 {
    aggregated + value
}

fn parallel_fold(num_threads: usize, inputs_iter: impl ConcurrentIter<Item = u64>) -> u64 {
    let inputs = &inputs_iter;
    let mut global_result = 0u64;

    std::thread::scope(|s| {
        let handles: Vec<_> = (0..num_threads)
            .map(|_| s.spawn(move || inputs.values().map(compute).fold(0u64, fold)))
            .collect();

        for h in handles {
            let thread_result = h.join().expect("o");
            global_result = fold(global_result, thread_result);
        }
    });
    global_result
}

// test
for num_threads in [1, 2, 4, 8] {
    let values = (0..1024).map(|x| 2 * x);
    assert_eq!(
        parallel_fold(num_threads, values.into_con_iter()),
        1023 * 1024
    );
}
```

Higher level functions may provide further simplifications. For instance, the `handles` above could also be collected as follows using the concurrent `fold` method:

```rust ignore
fn map_fold(aggregated: u64, input: u64) -> u64 {
    fold(aggregated, compute(input))
}

// ...

let handles: Vec<_> = (0..num_threads)
    .map(|_| s.spawn(move || inputs.fold(1, 0u64, map_fold)))
    .collect();
```

Parallel map can also be implemented by merging returned transformed collections, such as vectors. Especially for larger data types, a more efficient approach could be to pair `ConcurrentIter` with a concurrent collection such as [`orx_concurrent_bag::ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) which allows to efficiently collect results concurrently without copies.

```rust
use orx_concurrent_iter::*;
use orx_concurrent_bag::*;

type Input = u64;
type Output = [u64; 1];

fn map(input: Input) -> Output {
    [input]
}

fn parallel_map(
    num_threads: usize,
    inputs_iter: impl ConcurrentIter<Item = u64>,
) -> SplitVec<Output> {
    let output_bag = ConcurrentBag::new();
    let outputs = &output_bag;
    let inputs = &inputs_iter;

    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(move || {
                for output in inputs.values().map(map) {
                    outputs.push(output);
                }
            });
        }
    });
    output_bag.into_inner()
}

// test
for num_threads in [1, 2, 4, 8] {
    let inputs = (0..1024).map(|x| 2 * x);
    let outputs = parallel_map(num_threads, inputs.into_con_iter());
    assert_eq!(1024, outputs.len())
}
```

Note that due to parallelization, `outputs` is not guaranteed to be in the same order as `inputs`. In order to preserve the order of the input in the output, [`ConcurrentIter::ids_and_values`] method can be used, rather than the `values`, to get indices of values while iterating (see the explanation in the next section). Similarly, `ConcurrentBag` can be replaced with [`orx_concurrent_bag::ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag) to guarantee that the order is preserved.

### Iteration with Indices

In a single-threaded regular `Iterator`, values can be paired up with their indices easily by calling `enumerate` on the iterator. We can also call `inputs.values().enumerate()`; however, this would have a different meaning in a multi-threaded execution. It would pair up the values with the indices of the iteration local to that thread. In other words, the first value of every thread will be zero.

Actual iteration index of values can be obtained simply by using [`ConcurrentIter::ids_and_values`] instead of [`ConcurrentIter::values`].

```rust
use orx_concurrent_iter::*;

fn get_all_indices(num_threads: usize, inputs_iter: impl ConcurrentIter) -> Vec<usize> {
    let mut all_indices = vec![];
    let inputs = &inputs_iter;

    std::thread::scope(|s| {
        let handles: Vec<_> = (0..num_threads)
            .map(move |_| {
                s.spawn(|| {
                    inputs
                        .ids_and_values()
                        .map(|(i, _value)| i)
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        for handle in handles {
            let indices_for_thread = handle.join().expect("-");
            all_indices.extend(indices_for_thread);
        }
    });

    all_indices.sort();
    all_indices
}

// test
let inputs = ['a', 'b', 'c', 'd', 'e', 'f'];
let indices = get_all_indices(4, inputs.into_con_iter());
assert_eq!(indices, [0, 1, 2, 3, 4, 5]);
```

Notice that:
* `indices_for_thread` vectors collected for each thread are internally in ascending order. In other words, each thread receives elements in order.
* However, `indices_for_thread` vectors have gaps corresponding to indices of elements pulled by other threads.
* No index appears more than once in any of the `indices_for_thread` vectors.
* And union of these vectors give the indices from 0 to `n-1` where n is the number of yielded elements.

### Iteration in Chunks

In the default iteration using `for` together with `values` and `ids_and_values` methods, the threads pull elements one by one. Note that these iterators internally call `ConcurrentIter::next` and `ConcurrentIter::next_id_and_value` methods, respectively.

Further, it is also possible to iterate in chunks with `ConcurrentIter::next_chunk` and `ExactSizeConcurrentIter::next_exact_chunk` methods. These methods differ from `next` and `next_id_and_value` in the following:
* They receive the `chunk_size` parameter.
* They return an `Iterator` or `ExactSizeIterator` which yields the next `chunk_size` or fewer **consecutive** elements.
* Further, they additionally return the index of the first element that the returned iterator will yield. Note that the index of the remaining elements can be known since the iterator will return consecutive elements.

The advantage of `ExactSizeConcurrentIter` over `ConcurrentIter`, or `next_exact_chunk` over `next_chunk` is that it returns an iterator with an exact known size, which allows to return `None` when the iterator is empty. This has ergonomic benefits such as allowing `if let Some` or `while let Some` as demonstrated below. However, as expected, sources only with known lengths allow for it.
 
```rust
use orx_concurrent_iter::*;

fn lag(millis: u64) {
    std::thread::sleep(std::time::Duration::from_millis(millis));
}

let inputs = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
let characters = &inputs.con_iter();

let [first, second] = std::thread::scope(|s| {
    let first = s.spawn(move || {
        let mut chars: Vec<char> = vec![];
        while let Some(chunk) = characters.next_chunk(3) {
            chars.extend(chunk.values.copied());
            lag(100);
        }
        chars
    });

    let second = s.spawn(move || {
        lag(50);
        let mut chars: Vec<char> = vec![];
        while let Some(chunk) = characters.next_chunk(3) {
            chars.extend(chunk.values.copied());
            lag(100);
        }
        chars
    });

    let first = first.join().expect("-");
    let second = second.join().expect("o");
    [first, second]
});

// Events in chronological order:
// * first pulls 3 consecutive elements [a, b, c]
// * second pulls 3 consecutive elements [d, e, f]
// * first pulls remaining 2 consecutive elements [h, i]

assert_eq!(first, ['a', 'b', 'c', 'g', 'h']);
assert_eq!(second, ['d', 'e', 'f']);
```

## Traits and Implementors

As discussed so far, the trait defining types that can be safely be iterated concurrently by multiple threads is `ConcurrentIter`.

Further, there are two traits which define types that can provide a `ConcurrentIter`.
* A `ConcurrentIterable` type implements the **`con_iter(&self)`** method which returns a concurrent iterator without consuming the type itself.
* On the other hand, types implementing `IntoConcurrentIter` trait has the **`into_con_iter(self)`** method which consumes and converts the type into a concurrent iterator. Additionally there exists `IterIntoConcurrentIter` trait which is functionally identical to `IntoConcurrentIter` and only implemented by regular iterators, separated only to allow for special implementations for vectors and arrays.

The following table summarizes the implementations of the standard types in this crate.

| Type | ConcurrentIterable <br/> `con_iter()` element type | IntoConcurrentIter <br/> `into_con_iter()` element type | Cloned <br/> `con_iter().cloned()` |
|---|---|---|---|
| `&'a [T]` | `&'a T` | `&'a T` | `T` |
| `Range<Idx>` | `Idx` | `Idx` | |
| `Vec<T>` | `&T` | `T` | |
| `[T; N]` | `&T` | `T` | |
| `Iter: Iterator<Item = T>` | - | `T` | |


## License

This library is licensed under MIT license. See LICENSE for details.
