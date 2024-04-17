#![allow(dead_code)]

use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::*;
use std::ops::Add;

pub(crate) const ATOMIC_TEST_LEN: usize = 512;

pub(crate) const ATOMIC_FETCH_N: [usize; 8] = [
    1,
    2,
    4,
    8,
    ATOMIC_TEST_LEN / 2,
    ATOMIC_TEST_LEN,
    ATOMIC_TEST_LEN + 1,
    ATOMIC_TEST_LEN * 2,
];

pub(crate) fn atomic_fetch_one<A>(iter: A)
where
    A: AtomicIter,
    A::Item: Add<usize, Output = usize>,
{
    assert_eq!(0, iter.counter().current());

    let mut i = 0;
    while let Some(next) = iter.fetch_one() {
        let value = next.value + 0usize;
        assert_eq!(value, i);
        i += 1;
        assert_eq!(i, iter.counter().current());
    }
}

pub(crate) fn atomic_fetch_n<A>(iter: A, n: usize)
where
    A: AtomicIter,
    A::Item: Add<usize, Output = usize>,
{
    assert_eq!(0, iter.counter().current());

    let mut i = 0;
    let mut has_more = true;

    while has_more {
        has_more = false;
        let next_id_and_chunk = iter.fetch_n(n);
        let begin_idx = next_id_and_chunk.begin_idx();
        for (j, value) in next_id_and_chunk.values().enumerate() {
            let value = value + 0usize;
            assert_eq!(value, begin_idx + j);
            assert_eq!(value, i);

            i += 1;

            has_more = true;
        }
    }
}

pub(crate) fn atomic_exact_fetch_one<A>(iter: A)
where
    A: AtomicIterWithInitialLen,
    A::Item: Add<usize, Output = usize>,
{
    let mut remaining = ATOMIC_TEST_LEN;

    assert!(!iter.is_empty());
    assert_eq!(iter.len(), remaining);

    while iter.fetch_one().is_some() {
        remaining -= 1;
        assert_eq!(iter.len(), remaining);
    }

    assert_eq!(iter.len(), 0);
    assert!(iter.is_empty());
}

pub(crate) fn atomic_exact_fetch_n<A>(iter: A, n: usize)
where
    A: AtomicIterWithInitialLen,
    A::Item: Add<usize, Output = usize>,
{
    let mut remaining = ATOMIC_TEST_LEN;

    assert!(!iter.is_empty());
    assert_eq!(iter.len(), remaining);

    let mut has_more = true;
    while has_more {
        has_more = false;

        let next_id_and_chunk = iter.fetch_n(n);
        if next_id_and_chunk.values().next().is_some() {
            has_more = true;
        }

        if n > remaining {
            remaining = 0;
        } else {
            remaining -= n;
        }

        assert_eq!(iter.len(), remaining);
    }

    assert_eq!(iter.len(), 0);
    assert!(iter.is_empty());
}

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

pub(crate) fn test_ids_and_values<C: ConcurrentIter>(num_threads: usize, len: usize, con_iter: C)
where
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