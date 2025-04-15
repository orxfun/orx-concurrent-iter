use orx_concurrent_iter::*;
use test_case::test_matrix;

#[cfg(not(miri))]
const LEN: usize = 1024;
#[cfg(miri)]
const LEN: usize = 67;

#[test]
fn con_iter() {
    let array = ['a', 'b', 'c'];

    let con_iter = array.into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = ['a', 'b', 'c', 'd'];

    let iter = values.into_con_iter();
    assert_eq!(iter.try_get_len(), Some(4));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(3));

    let mut puller = iter.chunk_puller(2);
    _ = puller.pull();
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let mut array = [0; LEN];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.into_con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, &i);
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let mut array = [0; LEN];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.into_con_iter();

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
        assert_eq!(x, &(114 + i));
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let mut array = [0; LEN];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.into_con_iter().cloned();

    std::thread::scope(|s| {
        s.spawn(|| {
            let mut puller = con_iter.chunk_puller(21);
            if let Some(batch) = puller.pull() {
                for _ in batch {}
            }

            let mut puller = con_iter.chunk_puller(20);
            if let Some(batch) = puller.pull() {
                for _ in batch.take(12) {}
            }
        });
    });

    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN - 41);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, 41 + i);
    }
}

#[test]
fn into_seq_iter_doc() {
    let mut array = [0; LEN];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.into_con_iter().cloned();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..22 {
                _ = con_iter.next();
            }

            let mut puller = con_iter.chunk_puller(32);
            let _chunk = puller.pull().unwrap();
        });
    });

    let num_used = 22 + 32;

    // converts the remaining elements into a sequential iterator
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN - num_used);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, num_used + i);
    }
}

#[test]
fn into_seq_iter_not_used() {
    let mut values = [0; LEN];
    for (i, x) in values.iter_mut().enumerate() {
        *x = i;
    }
    let iter = values.into_con_iter().cloned().into_seq_iter();
    let result: Vec<_> = iter.collect();

    assert_eq!(result, values);
}

#[test_matrix([1, 10, 100])]
fn into_seq_iter_used(take: usize) {
    let mut values = [0; LEN];
    for (i, x) in values.iter_mut().enumerate() {
        *x = i;
    }

    let iter = values.into_con_iter().cloned();
    for _ in 0..take {
        _ = iter.next();
    }
    let result: Vec<_> = iter.into_seq_iter().collect();

    let mut iter = values.into_iter();
    for _ in 0..take {
        _ = iter.next();
    }
    let expected: Vec<_> = iter.collect();

    assert_eq!(result, expected);
}

#[test_matrix([1, 10, 100])]
fn buffered(chunk_size: usize) {
    let mut values = [0; LEN];
    for (i, x) in values.iter_mut().enumerate() {
        *x = 100 + i;
    }
    let iter = values.into_con_iter().cloned();
    let mut puller = iter.chunk_puller(chunk_size);

    let mut current = 100;
    while let Some(chunk) = puller.pull() {
        for value in chunk {
            assert_eq!(value, current);
            current += 1;
        }
    }

    assert_eq!(current, 100 + LEN);
}
