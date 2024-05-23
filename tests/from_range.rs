use orx_concurrent_iter::*;
use test_case::test_matrix;

#[test]
fn con_iter() {
    let values = 42..45;

    let con_iter: ConIterOfRange<_> = values.con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);

    let con_iter: ConIterOfRange<_> = values.clone().con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);

    let con_iter: ConIterOfRange<_> = values.into_exact_con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), Some(44));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn into_con_iter() {
    use orx_concurrent_iter::IterIntoConcurrentIter;

    let values = 42..45;

    let con_iter: ConIterOfIter<_, _> = values.into_iter().take(2).into_con_iter();
    assert_eq!(con_iter.next(), Some(42));
    assert_eq!(con_iter.next(), Some(43));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn exact_len() {
    let values = 42..45;
    assert_eq!(3, values.exact_len());
}

#[test]
fn len() {
    let values = 42..46;

    let iter = values.con_iter();
    assert_eq!(iter.len(), 4);
    assert_eq!(iter.try_get_len(), Some(4));

    _ = iter.next();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.try_get_len(), Some(3));

    _ = iter.next_chunk(2);
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.next();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let range = 0..1024;
    let con_iter = range.con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, i);
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let range = 0..1024;
    let con_iter = range.con_iter();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..114 {
                _ = con_iter.next();
            }
        });
    });

    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024 - 114);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, 114 + i);
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let range = 0..1024;
    let con_iter = range.con_iter();

    std::thread::scope(|s| {
        s.spawn(|| {
            if let Some(batch) = con_iter.next_chunk(44) {
                for _ in batch.values {}
            }

            if let Some(batch) = con_iter.next_chunk(33) {
                for _ in batch.values.take(22) {}
            }
        });
    });

    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024 - 44 - 33);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, 44 + 33 + i);
    }
}

#[test]
fn into_seq_iter_doc() {
    let range = 0..1024;
    let con_iter = range.con_iter();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..42 {
                _ = con_iter.next();
            }

            let mut buffered = con_iter.buffered_iter(32);
            let _chunk = buffered.next().unwrap();
        });
    });

    let num_used = 42 + 32;

    // converts the remaining elements into a sequential iterator
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024 - num_used);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, num_used + i);
    }
}

#[test_matrix([0, 42], [0, 1, 42, 43, 100])]
fn into_seq_iter_not_used(begin: usize, end: usize) {
    let range = || begin..end;
    let iter = range().con_iter().into_seq_iter();
    assert_eq!(iter, range());
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
