use crate::{
    iter::{
        atomic_counter::AtomicCounter,
        atomic_iter::{AtomicIter, AtomicIterWithInitialLen},
    },
    NextChunk, NextManyExact,
};
use std::cmp::Ordering;

/// A concurrent iterator over a slice yielding references to the elements.
#[derive(Debug)]
pub struct ConIterOfSlice<'a, T: Send + Sync> {
    slice: &'a [T],
    counter: AtomicCounter,
}

impl<'a, T: Send + Sync> ConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            counter: AtomicCounter::new(),
        }
    }

    /// Returns a reference to the underlying slice.
    pub fn as_slice(&self) -> &'a [T] {
        self.slice
    }
}

impl<'a, T: Send + Sync> From<&'a [T]> for ConIterOfSlice<'a, T> {
    /// Creates a concurrent iterator for the given `slice`.
    fn from(slice: &'a [T]) -> Self {
        Self::new(slice)
    }
}

impl<'a, T: Send + Sync> Clone for ConIterOfSlice<'a, T> {
    fn clone(&self) -> Self {
        Self {
            slice: self.slice,
            counter: self.counter.clone(),
        }
    }
}

impl<'a, T: Send + Sync> AtomicIter for ConIterOfSlice<'a, T> {
    type Item = &'a T;

    fn counter(&self) -> &AtomicCounter {
        &self.counter
    }

    fn get(&self, item_idx: usize) -> Option<Self::Item> {
        self.slice.get(item_idx)
    }

    fn fetch_n(&self, n: usize) -> impl NextChunk<Self::Item> {
        self.fetch_n_with_exact_len(n)
    }
}

impl<'a, T: Send + Sync> AtomicIterWithInitialLen for ConIterOfSlice<'a, T> {
    fn initial_len(&self) -> usize {
        self.slice.len()
    }

    fn fetch_n_with_exact_len(
        &self,
        n: usize,
    ) -> NextManyExact<Self::Item, impl ExactSizeIterator<Item = Self::Item>> {
        let begin_idx = self.counter.fetch_and_add(n);

        let values = match begin_idx.cmp(&self.slice.len()) {
            Ordering::Less => {
                let end_idx = (begin_idx + n).min(self.slice.len()).max(begin_idx);
                let values = self.slice[begin_idx..end_idx].iter();
                values
            }
            _ => [].iter(),
        };

        NextManyExact { begin_idx, values }
    }
}

unsafe impl<'a, T: Send + Sync> Sync for ConIterOfSlice<'a, T> {}

unsafe impl<'a, T: Send + Sync> Send for ConIterOfSlice<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        iter::{
            atomic_iter::tests::{
                atomic_exact_fetch_n, atomic_exact_fetch_one, atomic_fetch_n, atomic_fetch_one,
                ATOMIC_FETCH_N, ATOMIC_TEST_LEN,
            },
            con_iter::tests::{test_ids_and_values, test_values},
        },
        ConcurrentIter, IntoConcurrentIter,
    };
    use test_case::test_matrix;

    #[test]
    fn new() {
        let values = ['a', 'b', 'c'];
        let slice = values.as_slice();

        let con_iter = ConIterOfSlice::new(slice);
        let vec = con_iter.as_slice().to_vec();
        assert_eq!(slice, &vec);
    }

    #[test]
    fn from() {
        let values = ['a', 'b', 'c'];
        let slice = values.as_slice();

        let con_iter: ConIterOfSlice<_> = slice.into();
        let vec = con_iter.as_slice().to_vec();
        assert_eq!(slice, &vec);
    }

    #[test]
    fn debug() {
        let values = ['a', 'b', 'c'];
        let con_iter: ConIterOfSlice<_> = values.as_slice().into();

        assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfSlice { slice: ['a', 'b', 'c'], counter: AtomicCounter { current: 0 } }"
        );

        assert_eq!(con_iter.next(), Some(&'a'));

        assert_eq!(
            format!("{:?}", con_iter),
            "ConIterOfSlice { slice: ['a', 'b', 'c'], counter: AtomicCounter { current: 1 } }"
        );
    }

    #[test]
    fn as_slice() {
        let values = ['a', 'b', 'c'];
        let slice = values.as_slice();

        let con_iter: ConIterOfSlice<_> = slice.into();

        assert_eq!(con_iter.next(), Some(&'a'));

        let vec = con_iter.as_slice().to_vec();
        assert_eq!(slice, &vec);
    }

    #[test]
    fn clone() {
        let values = ['a', 'b', 'c'];
        let slice = values.as_slice();

        let con_iter: ConIterOfSlice<_> = slice.into();

        assert_eq!(con_iter.next(), Some(&'a'));
        assert_eq!(1, con_iter.counter().current());

        let clone = con_iter.clone();
        assert_eq!(1, clone.counter().current());

        assert_eq!(clone.next(), Some(&'b'));
        assert_eq!(clone.next(), Some(&'c'));
        assert_eq!(3, clone.counter().current());

        assert_eq!(clone.next(), None);
        assert_eq!(4, clone.counter().current());

        assert_eq!(clone.next(), None);
        assert_eq!(5, clone.counter().current());

        assert_eq!(1, con_iter.counter().current());
    }

    #[test]
    fn atomic() {
        let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
        atomic_fetch_one(ConIterOfSlice::new(values.as_slice()));
        for n in ATOMIC_FETCH_N {
            atomic_fetch_n(ConIterOfSlice::new(values.as_slice()), n);
        }
    }

    #[test]
    fn atomic_exact() {
        let values: Vec<_> = (0..ATOMIC_TEST_LEN).collect();
        atomic_exact_fetch_one(ConIterOfSlice::new(values.as_slice()));
        for n in ATOMIC_FETCH_N {
            atomic_exact_fetch_n(ConIterOfSlice::new(values.as_slice()), n);
        }
    }

    #[test_matrix(
        [1, 2, 8],
        [1, 2, 8, 64, 1025, 5483]
    )]
    fn ids_and_values(num_threads: usize, len: usize) {
        let values: Vec<_> = (0..len).collect();
        let slice = values.as_slice();
        test_values(num_threads, len, slice.into_con_iter());
        test_ids_and_values(num_threads, len, slice.into_con_iter());
    }
}
