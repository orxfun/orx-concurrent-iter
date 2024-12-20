use orx_concurrent_iter::*;
use test_case::test_matrix;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.con_iter();
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

    _ = iter.next_chunk(2);
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let mut array = [0; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, &i);
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let mut array = [0; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.con_iter();

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
        assert_eq!(x, &(114 + i));
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let mut array = [0; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.con_iter().cloned();

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
    let mut array = [0; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.con_iter().cloned();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..42 {
                _ = con_iter.next();
            }

            let mut buffered = con_iter.buffered_iter_x(32);
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

#[test]
fn into_seq_iter_not_used() {
    let mut values = [0; 1024];
    for (i, x) in values.iter_mut().enumerate() {
        *x = i;
    }
    let iter = values.con_iter().cloned().into_seq_iter();
    let result: Vec<_> = iter.collect();

    assert_eq!(result, values);
}

#[test_matrix([1, 10, 100])]
fn into_seq_iter_used(take: usize) {
    let mut values = [0; 1024];
    for (i, x) in values.iter_mut().enumerate() {
        *x = i;
    }

    let iter = values.con_iter().cloned();
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
    let mut values = [0; 1024];
    for (i, x) in values.iter_mut().enumerate() {
        *x = 100 + i;
    }
    let iter = values.con_iter().cloned();
    let mut buffered = iter.buffered_iter_x(chunk_size);

    let mut current = 100;
    while let Some(chunk) = buffered.next() {
        for value in chunk.values {
            assert_eq!(value, current);
            current += 1;
        }
    }

    assert_eq!(current, 100 + 1024);
}
