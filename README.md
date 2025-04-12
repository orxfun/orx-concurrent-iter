# orx-concurrent-iter

[![orx-concurrent-iter crate](https://img.shields.io/crates/v/orx-concurrent-iter.svg)](https://crates.io/crates/orx-concurrent-iter)
[![orx-concurrent-iter crate](https://img.shields.io/crates/d/orx-concurrent-iter.svg)](https://crates.io/crates/orx-concurrent-iter)
[![orx-concurrent-iter documentation](https://docs.rs/orx-concurrent-iter/badge.svg)](https://docs.rs/orx-concurrent-iter)

A thread-safe and ergonomic concurrent iterator trait and efficient lock-free implementations.

## Examples

### Basic Usage

A [`ConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html) can be safely shared among threads and iterated over concurrently. As expected, it will yield each element only once and in order. The yielded elements will be shared among the threads which concurrently iterates based on first come first serve. In other words, threads concurrently pull remaining elements from the iterator.

```
use orx_concurrent_iter::*;
use std::fmt::Debug;

fn fake_work<T: Debug>(_x: T) {
    std::thread::sleep(std::time::Duration::from_nanos(10));
}

/// `process` elements of `iter` concurrently using `num_threads` threads
fn process_concurrently<T, I, F>(process: &F, num_threads: usize, iter: I)
where
    T: Send + Sync,
    F: Fn(T) + Send + Sync,
    I: ConcurrentIter<Item = T>,
{
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                // concurrently iterate over values in a `for` loop
                for value in iter.values() {
                    process(value);
                }
            });
        }
    });
}

/// executes `fake_work` concurrently on all elements of the `concurrent_iter`
fn run<T: Send + Sync + Debug>(concurrent_iter: impl ConcurrentIter<Item = T>) {
    process_concurrently(&fake_work, 8, concurrent_iter)
}

// non-consuming iteration over references
let names: [String; 3] = [
    String::from("foo"),
    String::from("bar"),
    String::from("baz"),
];
run::<&String>(names.con_iter());

let values: Vec<i32> = (0..8).map(|x| 3 * x + 1).collect();
run::<&i32>(values.con_iter());

let slice: &[i32] = values.as_slice();
run::<&i32>(slice.con_iter());
run::<i32>(slice.con_iter().cloned());

// consuming iteration over values
run::<i32>(values.into_con_iter());

// any Iterator into ConcurrentIter
let values: Vec<i32> = (0..1024).collect();

let evens = values.iter().filter(|x| *x % 2 == 0);
run::<&i32>(evens.into_con_iter());

let evens = values.iter().filter(|x| *x % 2 == 0);
run::<i32>(evens.into_con_iter().cloned());

let iter_val = values
    .iter()
    .filter(|x| *x % 2 == 0)
    .map(|x| (7 * x + 3) as usize)
    .skip(2)
    .take(5);
run::<usize>(iter_val.into_con_iter());
```

### Ways to Iterate

A **ConcurrentIter** implements the [`next`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIterX.html#tymethod.next) method, which is the concurrent counterpart of **next** method of the regular Iterators.

Therefore, the usage of a concurrent iterator is almost equivalent to that of a regular **Iterator**. Slight differences are explained in the following example.

```
use orx_concurrent_iter::*;

fn process_one_by_one<T, I, F>(process: &F, num_threads: usize, iter: &I)
where
    T: Send + Sync,
    F: Fn(T) + Send + Sync,
    I: ConcurrentIter<Item = T>,
{
    std::thread::scope(|s| {
        for i in 0..num_threads {
            s.spawn(move || {
                match i % 4 {
                    0 => {
                        // pull values 1 by 1 (for)
                        for value in iter.values() {
                            process(value);
                        }
                    }
                    1 => {
                        // pull values 1 by 1 (while let)
                        while let Some(value) = iter.next() {
                            process(value);
                        }
                    }
                    2 => {
                        // pull values and corresponding index 1 by 1 (for)
                        for (idx, value) in iter.ids_and_values() {
                            println!("idx = {}", idx);
                            process(value);
                        }
                    }
                    _ => {
                        // pull values and corresponding index 1 by 1 (while let)
                        while let Some(x) = iter.next_id_and_value() {
                            println!("idx = {}", x.idx);
                            process(x.value);
                        }
                    }
                }
            });
        }
    });
}

fn process_in_chunks<T, I, F>(process: &F, num_threads: usize, iter: &I, chunk_size: usize)
where
    T: Send + Sync,
    F: Fn(T) + Send + Sync,
    I: ConcurrentIter<Item = T>,
{
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                // pull values in chunks of `chunk_size`
                let mut chunk_iter = iter.buffered_iter(chunk_size);
                while let Some(chunk) = chunk_iter.next() {
                    assert!(chunk.values.len() <= chunk_size);

                    for (i, value) in chunk.values.enumerate() {
                        let idx = chunk.begin_idx + i;
                        println!("idx = {}", idx);
                        process(value);
                    }
                }
            });
        }
    });
}

let process = |x| {
    println!("value = {}", x);
};

let vec: Vec<_> = (0..1024).collect();
process_one_by_one(&process, 8, &vec.con_iter().copied());

process_in_chunks(&process, 8, &vec.into_con_iter(), 64);
```

* **for** and **while let** loops of `process_one_by_one` demonstrate the most basic usage where threads will pull the next element of the iterator whenever they complete processing the prior element.
* Note that each thread will pull different elements at different positions of the iterator depending on how fast they finish the execution of the task inside the loop. Therefore, an `enumerate` call inside the thread, or counting the pulled elements by that particular thread, does **not** provide the index of the element in the original data source. **ConcurrentIter** additionally provides the original index with [`ids_and_values`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html#method.ids_and_values) or [`next_id_and_value`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html#tymethod.next_id_and_value) methods.
* When working with large collections while the work to be done inside the loop is too small, pulling elements 1-by-1 might be suboptimal. In such cases, a better idea is to pull elements in chunks. In `process_in_chunks`, we create a buffered chunk iterator with [`buffered_iter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html#method.buffered_iter) method. This iterator pulls **chunk_size consecutive** elements (or less if not enough left) at each **next** call. Note that **chunk** returned by the **next** method is an **ExactSizeIterator** with a known length.
* While iterating in chunks, we can still access the original index of the elements. **chunk.begin_idx** represents the original index of the first element of the returned **chunk.values** iterator. Note that **chunk.values** is always non-empty with at least one element; otherwise, **next** returns None. Further, recall that the chunk iterator contains consecutive elements. Hence, we can get the original indices of all elements by combining **chunk.begin_idx** with the local indices of the current **chunk.values** iterator, as demonstrated with `let idx = chunk.begin_idx + i`.


### Parallel Fold

Considering the elements of the iteration as inputs of a process, **ConcurrentIter** conveniently allows distribution of tasks to multiple threads. See below a parallel fold implementation using the concurrent iterator.

```
use orx_concurrent_iter::*;

fn compute(input: u64) -> u64 {
    input * 2
}

fn fold(aggregated: u64, value: u64) -> u64 {
    aggregated + value
}

fn par_fold(num_threads: usize, inputs: impl ConcurrentIter<Item = u64>) -> u64 {
    std::thread::scope(|s| {
        (0..num_threads)
            .map(|_| s.spawn(|| inputs.values().map(compute).fold(0u64, fold)))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|x| x.join().expect("-_-"))
            .fold(0u64, fold)
    })
}

// validate
for num_threads in [1, 2, 4, 8] {
    let values = (0..1024).map(|x| 2 * x);
    let par_result = par_fold(num_threads, values.into_con_iter());
    assert_eq!(par_result, 2 * 1023 * 1024);
}
```

Notes on the implementation:
* Concurrent iterator allows for a simple 7-line parallel fold implementation.
* Parallel fold operation is defined as two fold operations.
  * The first `.map(_).fold(_)` defines the parallel fold operation executed by `num_threads` threads. Each thread returns its own aggregated result.
  * The second `map(_).fold(_)` defines the final sequential fold operation executed to fold over the `num_threads` results obtained by each thread.

### Parallel Map

Parallel map can also be implemented by merging returned transformed collections, such as vectors. Especially for larger data types, a more efficient approach could be to pair **ConcurrentIter** with a concurrent collection such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) which allows to efficiently collect results concurrently without copies.

```
use orx_concurrent_iter::*;
use orx_concurrent_bag::*;

fn map(input: u64) -> String {
    input.to_string()
}

fn parallel_map(num_threads: usize, iter: impl ConcurrentIter<Item = u64>) -> SplitVec<String> {
    let outputs = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                for output in iter.values().map(map) {
                    outputs.push(output);
                }
            });
        }
    });
    outputs.into_inner()
}

// test
for num_threads in [1, 2, 4, 8] {
    let inputs = (0..1024).map(|x| 2 * x);
    let outputs = parallel_map(num_threads, inputs.into_con_iter());
    assert_eq!(1024, outputs.len());
}
```

Note that due to parallelization, **outputs** is not guaranteed to be in the same order as **inputs**. In order to preserve the order of the input in the output, iteration with indices can be used to sort the result accordingly. Alternative to post-sorting, **ConcurrentBag** can be replaced with [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag) to already collect in order.

### Parallel Find, A Little Communication Among Threads

As illustrated above, efficient parallel implementations of different methods are conveniently possible with **ConcurrentIter. There is only one bit of information implicitly shared among threads by the concurrent iterator: the elements left. In scenarios where we do not need to iterate over all elements, we can use this information to share a message among threads. We might call such cases as **early-return** scenarios. A common example is the `find` method, where we are looking for a matching element and we want to terminate the search as soon as we find one.

You may see a parallel implementation of the find method below.

```
use orx_concurrent_iter::*;

fn par_find<I, P>(iter: I, predicate: P, n_threads: usize) -> Option<(usize, I::Item)>
where
    I: ConcurrentIter,
    P: Fn(&I::Item) -> bool + Send + Sync,
{
    std::thread::scope(|s| {
        (0..n_threads)
            .map(|_| {
                s.spawn(|| {
                    for (i, x) in iter.ids_and_values() {
                        if predicate(&x) {
                            iter.skip_to_end();
                            return Some((i, x));
                        }
                    }
                    None
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|x| x.join().expect("(-)"))
            .min_by_key(|x| x.0)
    })
}

let mut names: Vec<_> = (0..8785).map(|x| x.to_string()).collect();
names[42] = "foo".to_string();

let result = par_find(names.con_iter(), |x| x.starts_with('x'), 4);
assert_eq!(result, None);

let result = par_find(names.con_iter(), |x| x.starts_with('f'), 4);
assert_eq!(result, Some((42, &"foo".to_string())));

names[43] = "foo_second_match".to_string();
let result = par_find(names.con_iter(), |x| x.starts_with('f'), 4);
assert_eq!(result, Some((42, &"foo".to_string())));
```

Notice that the parallel find implementation is in two folds:
* (parallel search) Inside each thread, we loop through the elements of the concurrent iterator and return the first value satisfying the `predicate` together with its index.
* (sequential wrap up) Since this is a parallel execution, we might end up receiving multiple matches from multiple threads. In the second part, we investigate the thread results and return the one with the minimum position index (`min_by_key(|x| x.0)`) since that is the element which appears first in the original iterator.

So far, this is straightforward and similar to the parallel fold implementation.

The difference; however, is the additional [`iter.skip_to_end()`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIterX.html#tymethod.skip_to_end) call. This call will immediately consume all remaining elements of the iterator.

After the **skip_to_end** call, any thread trying to advance the iterator will observe that the iterator is terminated. Hence, they will as well return as soon as they complete processing their last pulled element.

This establishes a very trivial communication among threads, which is critical in achieving efficiency in early return scenarios, as the find method. To demonstrate, assume the case we did **not** make the **iter.skip_to_end()** call in the above implementation.
* In the second example, the iterator has 8785 elements where there exists only one element satisfying the predicate, "foo" at position 42.
* One of the 4 threads used, say thread **A**, will find this element and return immediately.
* The other 3 threads will never see this element, since it is pulled by **A**. They will iterate over all remaining elements and will eventually return None.
* The final result will be correct. However, this implementation will evaluate all elements of the iterator regardless of where the first matching element is. This would be a very inefficient parallel implementation.

## Traits and Implementors

The trait defining types that can be safely be iterated concurrently by multiple threads is [`ConcurrentIterX`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIterX.html#).

[`ConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIter.html#) trait extends **ConcurrentIterX** by adding a guarantee that the elements will be yield in the correct order.


Further, there are two traits which define types that can provide concurrent iterators.

* A [`ConcurrentIterable`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIterable.html) type implements the [`con_iter(&self)`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.ConcurrentIterable.html#tymethod.con_iter) method which returns a concurrent iterator without consuming the type itself.
* On the other hand, types implementing [`IntoConcurrentIterX`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIterX.html) and [`IntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIter.html) traits have the consuming [`into_con_iter_x(self)`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIterX.html#tymethod.into_con_iter_x) and [`into_con_iter(self)`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIter.html#tymethod.into_con_iter) methods which convert the type into a concurrent iterator. Finally, there exists [`IterIntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IterIntoConcurrentIter.html) trait which is functionally identical to **IntoConcurrentIter** and only implemented by regular iterators.

The following table summarizes the implementations of the standard types in this crate.

| Type | ConcurrentIterable <br/> `con_iter()` element type | IntoConcurrentIter <br/> `into_con_iter()` element type |
|---|---|---|
| `&'a [T]` | `&'a T` | `&'a T` |
| `Range<Idx>` | `Idx` | `Idx` |
| `Vec<T>` | `&T` | `T` |
| `[T; N]` | `&T` | `T` |
| `Iter: Iterator<Item = T>` | - | `T` |

Finally, concurrent iterators having an element type which is a reference to a **Clone** or **Copy** type, have the [`cloned()`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoCloned.html#method.cloned) or [`copied()`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoCopied.html#method.copied) methods, allowing to iterate over cloned values.

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-concurrent-iter/issues/new) or create a PR.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
