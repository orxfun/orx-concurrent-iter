# orx-concurrent-iter

[![orx-concurrent-iter crate](https://img.shields.io/crates/v/orx-concurrent-iter.svg)](https://crates.io/crates/orx-concurrent-iter)
[![orx-concurrent-iter crate](https://img.shields.io/crates/d/orx-concurrent-iter.svg)](https://crates.io/crates/orx-concurrent-iter)
[![orx-concurrent-iter documentation](https://docs.rs/orx-concurrent-iter/badge.svg)](https://docs.rs/orx-concurrent-iter)

A thread-safe and ergonomic concurrent iterator trait and efficient lock-free implementations.

This crate focuses on enabling **ergonomic** concurrent programs without sacrificing **efficiency**.

> **no-std**: This is a **no-std** crate.

## A. Ergonomics

A [`ConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html) can be safely shared among threads using a shared reference; and multiple threads can iterate over it concurrently.

As expected, it will yield each element only once and in order.

### A.1. next and while let

Just like a regular iterator, a concurrent iterator has a [`next`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html#tymethod.next) method returning and `Option` of the element type.

```rust
use orx_concurrent_iter::*;
use core::time::Duration;

let vec = vec!['a', 'b', 'c'];
let con_iter = vec.con_iter(); // concurrent iterator

std::thread::scope(|s| {
    s.spawn(|| { // 1st thread
        let first = con_iter.next();
        assert_eq!(first, Some(&'a'));

        // make sure that 2nd thread pulls 'b' and 'c'
        std::thread::sleep(Duration::from_secs(2));

        assert_eq!(con_iter.next(), None);
    });

    s.spawn(|| { // 2nd thread
        // make sure that 1st thread pulls 'a' beforehand
        std::thread::sleep(Duration::from_secs(1));

        let second = con_iter.next();
        assert_eq!(second, Some(&'b'));

        let third = con_iter.next();
        assert_eq!(third, Some(&'c'));

        assert_eq!(con_iter.next(), None);
    });
});
```

The signature of the `next` method allows convenient `while let` loops. 

The following example demonstrates a straightforward **parallel processing** implementation, where 100 elements of the input data will be processed in parallel by 4 threads.

Note that the code does not look different than its sequential counterpart, except for the `spawn` calls.

```rust
use orx_concurrent_iter::*;

let num_threads = 4;

let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
let con_iter = data.con_iter();

let process = |_x: &String| { /* assume actual work */ };

std::thread::scope(|s| {
    for _ in 0..num_threads {
        s.spawn(|| {
            while let Some(value) = con_iter.next() {
                process(value);
            }
        });
    }
});
```

### A.2. Pullers (within-thread Iterators) and for

Although rust's `while let` loops are already very convenient, we cannot use `for` loops because a concurrent iterator is not a regular `Iterator`. 

However, we can create one or more **item pullers** from a concurrent iterator within multiple threads, and the [`ItemPuller`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/struct.ItemPuller.html) implements `Iterator`. 

Although we use a puller similar to a regular iterator; it is created from, connected to and pulls elements from a `ConcurrentIter`.

This trick first enables `for` loops.

```rust
use orx_concurrent_iter::*;

let num_threads = 4;

let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
let con_iter = data.con_iter();

let process = |_x: &String| { /* assume actual work */ };

std::thread::scope(|s| {
    for _ in 0..num_threads {
        s.spawn(|| {
            for value in con_iter.item_puller() {
                process(value);
            }
        });
    }
});
```

The actual benefit, however, is to enable the convenient and composable iterator methods in concurrent programs.

Consider the following **parallel_reduce** operation, which is a simple yet efficient parallelization of the [`reduce`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.reduce) method.

Notice that:

* The entire implementation is a chain of Iterator methods fitting in four lines.
* We compute the reduction in two folds:
  * We map each thread to the reduction of the elements that it pulls.
  * We ignore (filter_map) the results that are None; this can be observed when a thread cannot find any elements to pull.
  * Then, we reduce the reduced results returned by each thread to get the final result.

```rust
use orx_concurrent_iter::*;

fn parallel_reduce<T, F>(
    num_threads: usize,
    con_iter: impl ConcurrentIter<Item = T>,
    reduce: F,
) -> Option<T>
where
    T: Send + Sync,
    F: Fn(T, T) -> T + Send + Sync,
{
    std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| s.spawn(|| con_iter.item_puller().reduce(&reduce))) // reduce inside each thread
            .filter_map(|x| x.join().unwrap()) // join threads, ignore None's
            .reduce(&reduce) // reduce thread results to final result
    })
}

// test

let sum = parallel_reduce(8, (0..0).into_con_iter(), |a, b| a + b);
assert_eq!(sum, None);

let sum = parallel_reduce(8, (0..3).into_con_iter(), |a, b| a + b);
assert_eq!(sum, Some(3));

let n = 10_000;
let data: Vec<_> = (0..n).collect();
let sum = parallel_reduce(8, data.con_iter().copied(), |a, b| a + b);
assert_eq!(sum, Some(n * (n - 1) / 2));
```

## B. Efficiency

The concurrent iterator trait definition and lock-free implementations are designed to achieve high performance concurrent programs.

### B.1. Iteration in Chunks

A major difference of a concurrent iterator from a sequential iterator is the concurrent state to be maintained by atomic variables. Every time elements are pulled from the concurrent iterator, this state is updated. These updates can be considered as the overhead of the concurrency.

Whenever the task to be performed over each element of the iterator is large enough, the overhead will be negligible. In other cases, it is important to reduce the number of state updates in order to improve performance.

A straightforward way to achieve this is by pulling in chunks rather than one element at a time. This can be done by a [`ChunkPuller`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ChunkPuller.html).

> Note that puling in chunks does **not** mean cloning or moving the elements; it only refers to reserving multiple elements at once for a thread that calls the pull.

Consider the **parallel processing** example above, in which 4 threads would have pulled 100 elements in a total of 100 `next` calls. This means that the concurrent state would have been updated 100 times. The following version, on the other hand, will reach the `con_iter` only 10 times and at each access the pulling thread will reserve 10 consecutive elements.

```rust
use orx_concurrent_iter::*;

let num_threads = 4;

let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();
let con_iter = data.con_iter();

let process = |_x: &String| { /* assume actual work */ };

std::thread::scope(|s| {
    for _ in 0..num_threads {
        s.spawn(|| {
            let mut puller = con_iter.chunk_puller(10);
            while let Some(chunk) = puller.pull() {
                for value in chunk {
                    process(value);
                }
            }
        });
    }
});
```

This is a simple but a very important optimization technique in performance critical programs where the `process` is considerably small. 

However, notice that we now need to have nested iterators. This is the explicit version and gives us the ability to operate on the `chunk` whenever needed. When we only need to perform on individual elements, we can flatten the chunk puller. Similar to the `ItemPuller`, a [`FlattenedChunkPuller`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/struct.FlattenedChunkPuller.html) also implements Iterator, and hence, brings benefits of iterator composability to concurrent programs.

The following **parallel_reduce** implementation adds the iteration-by-chunks optimization to the previous version; while keeping the implementation simple, still fitting to four lines by flattening the chunk iterator to a regular Iterator.

```rust
use orx_concurrent_iter::*;

fn parallel_reduce<T, F>(
    num_threads: usize,
    chunk: usize,
    con_iter: impl ConcurrentIter<Item = T>,
    reduce: F,
) -> Option<T>
where
    T: Send + Sync,
    F: Fn(T, T) -> T + Send + Sync,
{
    std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| s.spawn(|| con_iter.chunk_puller(chunk).flattened().reduce(&reduce))) // reduce inside each thread
            .filter_map(|x| x.join().unwrap()) // join threads, ignore None's
            .reduce(&reduce) // reduce thread results to final result
    })
}

let n = 10_000;
let data: Vec<_> = (0..n).collect();
let sum = parallel_reduce(8, 64, data.con_iter().copied(), |a, b| a + b);
assert_eq!(sum, Some(n * (n - 1) / 2));
```

### B.2. Early Exit

We do not always want to iterate over all elements. In certain programs, we would like to abort iteration as soon as we achieve a certain goal. Sequential iterator's [`find`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.find) method is an example use case where we stop iteration as soon as an item satisfies a predicate.

In order to achieve this in a concurrent iteration, we can conveniently use the [`skip_to_end`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html#tymethod.skip_to_end) method. After any of the threads calls this method, succeeding pull calls will receive None, allowing the calling threads to early-exit.

This is demonstrated in the following **parallel_find** implementation:

* The thread that pulls "33" will immediately send feedback to the concurrent iterator to terminate.
* After this point, all pull or next calls from the concurrent iterator will return None. This will allow all other threads to immediately return None.

> It is possible that a couple of other threads pull "34" and "35, while the winner thread is testing "33" against the predicate. However, once they complete testing these elements, they will not find a new item and exit early.

```rust
use orx_concurrent_iter::*;

fn parallel_find<T, F>(
    num_threads: usize,
    con_iter: impl ConcurrentIter<Item = T>,
    predicate: F,
) -> Option<T>
where
    T: Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
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
```

## C. Any Iterator as a Concurrent Iterator

The [`IterIntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IterIntoConcurrentIter.html) trait allows to convert any generic sequential Iterator into a `ConcurrentIter`.

This makes it very convenient to safely share any iterator across threads using a shared reference as demonstrated below.

```rust
use orx_concurrent_iter::*;

let num_threads = 4;

let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();

// an arbitrary iterator
let iter = data
    .into_iter()
    .filter(|x| !x.starts_with('3'))
    .map(|x| format!("{x}!"));

// converted into a concurrent iterator and shared with multiple threads
let con_iter = iter.iter_into_con_iter();

let process = |_x: String| { /* assume actual work */ };

std::thread::scope(|s| {
    for _ in 0..num_threads {
        s.spawn(|| {
            while let Some(value) = con_iter.next() {
                assert!(!value.starts_with('3'));
                assert!(value.ends_with('!'));
                process(value);
            }
        });
    }
});
```

However, due to the fact that the provider is any iterator type (we lack any further useful information to optimize), pulling of elements from the iterator is synchronized. This means that:

* Whenever possible, it is always more efficient to create the concurrent iterator from the concrete type which does not require the abovementioned synchronization.
* When the `process` is large enough; i.e., when the heavy computation load is on the processing side rather than on pulling of the items, the impact of synchronization diminishes and we benefit from parallelization.

Benchmarks [benches/con_iter_of_iter.rs](https://github.com/orxfun/orx-concurrent-iter/blob/main/benches/con_iter_of_iter.rs) and [benches/con_iter_of_iter_short.rs](https://github.com/orxfun/orx-concurrent-iter/blob/main/benches/con_iter_of_iter_short.rs) aim to test the performance of concurrent iterators created from generic iterators. We observe that significant performance improvements can be achieved through parallelization by concurrent iterators despite the impact of synchronization.

| Process         | Input Size | Method                           | Computation Time (ms) |
| --------------- | ---------- | -------------------------------- | --------------------- |
| short `process` | 65536      | sequential                       | 4.65                  |
|                 |            | rayon                            | 15.71                 |
|                 |            | ConcurrentIter (chunk_size = 1)  | 15.72                 |
|                 |            | ConcurrentIter (chunk_size = 64) | **1.80**              |
| long `process`  | 65536      | sequential                       | 24.30                 |
|                 |            | rayon                            | 23.08                 |
|                 |            | ConcurrentIter (chunk_size = 1)  | 19.09                 |
|                 |            | ConcurrentIter (chunk_size = 64) | **7.27**              |


*Default parallel iterator of rayon created by the par_bridge is used which might use available threads. For the concurrent iterator tests, 8 threads are spawned for each test case.*

## D. Implementations

The following table lists current implementations of concurrent iterators provided in this crate.

| Type                    | ConcurrentIterable `con_iter` | IntoConcurrentIter `into_con_iter` | IterIntoConcurrentIter `iter_into_con_iter` |
|-------------------------|-------------------------------|------------------------------------|---------------------------------------------|
| `I: Iterator<Item = T>` |                               |                                    | `T`                                         |
| `&[T]`                  | `&T`                          | `&T`                               |                                             |
| `Vec<T>`                | `&T`                          | `T`                                |                                             |
| `VecDeque<T>`                | `&T`                          | `T`                                |                                             |
| `Range<T>`              | `T`                           | `T`                                |                                             |

The following are collections with known concurrent iterator implementations in their respective crates:

| Type                    | ConcurrentIterable `con_iter` | IntoConcurrentIter `into_con_iter` |
|-------------------------|-------------------------------|------------------------------------|
| [`SplitVec<T, G>`](https://crates.io/crates/orx-split-vec)  | `&T`                          | `T`                                |
| [`FixedVec<T>`](https://crates.io/crates/orx-fixed-vec)  | `&T`                          | `T`                                |

### Relation to orx_parallel

***Any collection or generator type that can create a `ConcurrentIter` can be efficiently parallelized using [`orx_parallel`](https://crates.io/crates/orx-parallel).***

Notice that straightforward implementations of several parallel computations are provided as examples throughout the documentation. They demonstrate that `ConcurrentIter` establishes the input side of parallel computation. There are two additional required pieces:

* Concurrent collections to write the results to.
  * Consider, for instance, the parallelized **map** operation followed by a **collect**. Each element of the concurrent iterator must be mapped to a value which must be then safely and efficiently written to the output.
* A parallel runner.
  * Concurrent iterators allow to set the chunk size, or even, pull chunks with dynamic chunks sizes that can be changed during the iteration depending on observations.
  * Similarly, degree of parallelization can be determined taking the input data and computation into account.
  * These decisions must be taken by a parallel runner.

Building on top of concurrent iterators, these missing pieces are implemented in the **orx_parallel** crate which allows for high performance and configurable parallel computations expressed as compositions of parallel iterator methods.

## E. Creating Concurrent Iterators

We can create concurrent iterators using two methods `con_iter` and `into_con_iter`. These methods belong to three traits which are explained below.

> In addition, a concurrent iterator can be created from a generic iterator using `iter_into_con_iter`; however, this is explained separately above and made explicit.


### E.1. [`IntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/1.30.0/orx_concurrent_iter/trait.IntoConcurrentIter.html)

This trait represents all types that can be consumed and converted into a concurrent iterator.

Concurrent counterpart of the standard [`IntoIterator`] trait.

For instance, `Vec<T>` implements `IntoIterator<Item = T>`; and it also implements `IntoConcurrentIter<Item = T>` method.

* `vec.into_iter()` consumes `vec` and returns an iterator yielding items of `T`.
* `vec.into_con_iter()` consumes `vec` and returns a concurrent iterator yielding items of `T`.

### E.2. [`ConcurrentIterable`](https://docs.rs/orx-concurrent-iter/1.30.0/orx_concurrent_iter/trait.ConcurrentIterable.html)

This trait represents all types that can repeatedly create concurrent iterators for its elements, without consuming the type.

Concurrent counterpart of the [`Iterable`](https://docs.rs/orx-iterable/latest/orx_iterable/trait.Iterable.html) trait of the [orx_iterable](https://crates.io/crates/orx-iterable) create.

For instance, `&Range<usize>` implements `Iterable<Item = usize>`; and it also implements `ConcurrentIterable<Item = usize>` method.

* `range.iter()` returns an iterator yielding items of `usize` without consuming `range`.
* `range.con_iter()` returns a concurrent iterator yielding items of `usize` without consuming `range`.

### E.3. [`ConcurrentCollection`](https://docs.rs/orx-concurrent-iter/1.30.0/orx_concurrent_iter/trait.ConcurrentCollection.html)

This trait represents all types that can both repeatedly create concurrent iterators for references of its elements, and also converted into a concurrent iterator of its elements.

Concurrent counterpart of the [`Collection`](https://docs.rs/orx-iterable/latest/orx_iterable/trait.Collection.html) trait of the [orx_iterable](https://crates.io/crates/orx-iterable) create.

`Vec<T>` both implements `Collection<Item = T>` and `ConcurrentCollection<Item = T>`.

* Create iterators of references without consuming the `vec`.
  * `vec.iter()` returns an iterator yielding items of `&T`.
  * `vec.con_iter()` returns a concurrent iterator yielding items of `&T`.
* Consume `vec` and convert it into an iterator.
  * `vec.into_iter()` consumes `vec` and returns an iterator yielding items of `T`.
  * `vec.into_con_iter()` consumes `vec` and returns a concurrent iterator yielding items of `T`.


## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-concurrent-iter/issues/new) or create a PR.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
