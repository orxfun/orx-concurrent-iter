use orx_concurrent_iter::{
    implementations::{ConIterOfIter, ConIterRange},
    *,
};
use std::cmp::Ordering;
use test_case::test_matrix;

#[cfg(not(miri))]
const LEN: usize = 1024;
#[cfg(miri)]
const LEN: usize = 153;

#[test]
fn con_iter() {
    let values = 42..45;

    let con_iter: ConIterRange<_> = values.con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);

    let con_iter: ConIterRange<_> = values.clone().con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn into_con_iter() {
    let values = 42..45;

    let con_iter: ConIterOfIter<_> = values.into_iter().take(2).iter_into_con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = 42..46;

    let iter = values.con_iter();
    assert_eq!(iter.try_get_len(), Some(4));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(3));

    _ = iter.chunk_puller(2).pull();
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let range = 0..LEN;
    let con_iter = range.con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, i);
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let range = 0..LEN;
    let con_iter = range.con_iter();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..114 {
                _ = con_iter.next();
            }
        });
    });

    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN - 114);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, 114 + i);
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let range = 0..LEN;
    let con_iter = range.con_iter();

    std::thread::scope(|s| {
        s.spawn(|| {
            if let Some(batch) = con_iter.chunk_puller(44).pull() {
                for _ in batch {}
            }

            if let Some(batch) = con_iter.chunk_puller(33).pull() {
                for _ in batch.take(22) {}
            }
        });
    });

    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN - 44 - 33);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, 44 + 33 + i);
    }
}

#[test]
fn into_seq_iter_doc() {
    let range = 0..LEN;
    let con_iter = range.con_iter();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..42 {
                _ = con_iter.next();
            }

            let mut buffered = con_iter.chunk_puller(32);
            let _chunk = buffered.pull().unwrap();
        });
    });

    let num_used = 42 + 32;

    // converts the remaining elements into a sequential iterator
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN - num_used);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, num_used + i);
    }
}

#[test_matrix([0, 42], [0, 1, 42, 43, 100])]
fn into_seq_iter_not_used(begin: usize, end: usize) {
    let range = || begin..end;
    let iter = range().con_iter().into_seq_iter();
    assert_eq!(iter.collect::<Vec<_>>(), range().collect::<Vec<_>>());
}

#[test_matrix([0, 42], [0, 1, 42, 43, 100], [1, 10, 100])]
fn into_seq_iter_used(begin: usize, end: usize, take: usize) {
    let range = || begin..end;

    let iter = range().con_iter();
    for _ in 0..take {
        _ = iter.next();
    }
    let remaining: Vec<_> = iter.into_seq_iter().collect();

    let vec: Vec<_> = range().collect();
    let mut iter = vec.into_iter();
    for _ in 0..take {
        _ = iter.next();
    }
    let expected: Vec<_> = iter.collect();

    assert_eq!(remaining, expected);
}

#[test_matrix([0, 42], [0, 1, 42, 43, 100], [1, 10, 100])]
fn buffered(begin: usize, end: usize, chunk_size: usize) {
    let iter = (begin..end).con_iter();
    let mut buffered = iter.chunk_puller(chunk_size);

    let mut current = begin;
    while let Some(chunk) = buffered.pull() {
        for value in chunk {
            assert_eq!(value, current);
            current += 1;
        }
    }

    match end.cmp(&begin) {
        Ordering::Less => assert_eq!(current, begin),
        _ => assert_eq!(current, end),
    }
}
