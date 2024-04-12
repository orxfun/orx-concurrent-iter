use super::wrappers::values::ConIterValues;
use crate::{next::Next, ConIterIdsAndValues, NextChunk, NextManyExact};

/// Trait defining a concurrent iterator with `next` and `next_id_and_chunk` methods which can safely be called my multiple threads concurrently.
pub trait ConcurrentIter: Send + Sync {
    /// Type of the items that the iterator yields.
    type Item: Send + Sync;

    /// Advances the iterator and returns the next value together with its enumeration index.
    ///
    /// Returns [None] when iteration is finished.
    fn next_id_and_value(&self) -> Option<Next<Self::Item>>;

    /// Advances the iterator `chunk_size` times and returns an iterator of at most `chunk_size` consecutive next values.
    /// Further, the beginning enumeration index of the yielded values is returned.
    ///
    /// This method:
    /// * returns an iterator of `chunk_size` elements if there exists sufficient elements left in the iteration, or
    /// * it might return an iterator of `m < chunk_size` elements if there exists only `m` elements left, or
    /// * it might return an empty iterator.
    ///
    /// This call would be equivalent to calling `next_id_and_value` method `chunk_size` times in a single-threaded execution.
    /// However, calling `next` method `chunk_size` times in a concurrent execution does not guarantee to return `chunk_size` consecutive elements.
    /// On the other hand, `next_id_and_chunk` guarantees that it returns consecutive elements, preventing any intermediate calls.
    fn next_chunk(&self, chunk_size: usize) -> impl NextChunk<Self::Item>;

    /// Advances the iterator and returns the next value.
    ///
    /// Returns [None] when iteration is finished.
    #[inline(always)]
    fn next(&self) -> Option<Self::Item> {
        self.next_id_and_value().map(|x| x.value)
    }

    /// Returns an `Iterator` over the values of elements of the concurrent iterator.
    ///
    /// The iterator's `next` method does nothing but call the `next`; this iterator is only to allow for using `for` loops directly.
    fn values(&self) -> ConIterValues<Self>
    where
        Self: Sized,
    {
        self.into()
    }

    /// Returns an `Iterator` over the ids and values of elements of the concurrent iterator.
    ///
    /// The iterator's `next` method does nothing but call the `next_id_and_value`; this iterator is only to allow for using `for` loops directly.
    fn ids_and_values(&self) -> ConIterIdsAndValues<Self>
    where
        Self: Sized,
    {
        self.into()
    }
}

/// A concurrent iterator that knows its exact length.
pub trait ExactSizeConcurrentIter: ConcurrentIter {
    /// Returns the exact remaining length of the concurrent iterator.
    fn len(&self) -> usize;

    /// Returns true if the iterator is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the next chunk with the requested `chunk_size`:
    /// * Returns `None` if there are no more elements to yield.
    /// * Returns `Some` of a [`crate::NextManyExact`] which contains the following information:
    ///   * `begin_idx`: the index of the first element to be yielded by the `values` iterator.
    ///   * `values`: an `ExactSizeIterator` with known `len` which is guaranteed to be positive and less than or equal to `chunk_size`.
    fn next_exact_chunk(
        &self,
        chunk_size: usize,
    ) -> Option<NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>>>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use orx_concurrent_bag::ConcurrentBag;
    use std::ops::Add;

    pub(crate) fn test_values<C: ConcurrentIter>(num_threads: usize, len: usize, con_iter: C)
    where
        C::Item: Add<usize, Output = usize>,
    {
        let collected = ConcurrentBag::new();
        let bag = &collected;
        let iter = &con_iter;

        std::thread::scope(|s| {
            for _ in 0..num_threads {
                s.spawn(move || {
                    for value in iter.values() {
                        bag.push(value + 0usize);
                    }
                });
            }
        });

        assert_eq!(collected.len(), len);

        let mut collected = collected.into_inner().to_vec();

        collected.sort();
        assert_eq!(collected, (0..len).collect::<Vec<_>>());
    }

    pub(crate) fn test_ids_and_values<C: ConcurrentIter>(
        num_threads: usize,
        len: usize,
        con_iter: C,
    ) where
        C::Item: Add<usize, Output = usize>,
    {
        let collected = ConcurrentBag::new();
        let bag = &collected;
        let iter = &con_iter;

        std::thread::scope(|s| {
            for _ in 0..num_threads {
                s.spawn(move || {
                    for (i, value) in iter.ids_and_values() {
                        bag.push((i, value + 0usize));
                    }
                });
            }
        });

        assert_eq!(collected.len(), len);

        let mut collected = collected.into_inner().to_vec();
        for (i, value) in &collected {
            assert_eq!(i, value);
        }

        collected.sort();
        assert_eq!(collected, (0..len).map(|x| (x, x)).collect::<Vec<_>>());
    }
}
