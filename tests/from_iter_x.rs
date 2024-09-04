use orx_concurrent_iter::*;
use test_case::test_matrix;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.iter().into_con_iter_x();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().take(2).into_con_iter_x();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().skip(1).into_con_iter_x();
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values
        .iter()
        .filter(|x| **x != 'a')
        .map(|x| x.to_string())
        .into_con_iter_x();
    assert_eq!(con_iter.next(), Some(String::from('b')));
    assert_eq!(con_iter.next(), Some(String::from('c')));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = ['a', 'b', 'c', 'd'];
    let iter = values.iter();

    let iter = iter.into_con_iter_x();
    assert_eq!(iter.try_get_len(), Some(4));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(3));

    _ = iter.next_chunk_x(2);
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.next_chunk_x(2);
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter_x();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, i.to_string());
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter_x();

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
        assert_eq!(x, (114 + i).to_string());
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter_x();

    std::thread::scope(|s| {
        s.spawn(|| {
            if let Some(batch) = con_iter.next_chunk_x(44) {
                for _ in batch {}
            }

            if let Some(batch) = con_iter.next_chunk_x(33) {
                for _ in batch.take(22) {}
            }
        });
    });

    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024 - 44 - 33);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, (44 + 33 + i).to_string());
    }
}

#[test]
fn into_seq_iter_doc() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter_x();

    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..42 {
                _ = con_iter.next();
            }

            let mut buffered = con_iter.buffered_iter_x(32);
            let _chunk = buffered.next_x().unwrap();
        });
    });

    let num_used = 42 + 32;

    // converts the remaining elements into a sequential iterator
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024 - num_used);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, (num_used + i).to_string());
    }
}

#[test_matrix([1, 8, 64, 1025, 5483])]
fn into_seq_iter_not_used(len: usize) {
    let values: Vec<_> = (100..(100 + len)).collect();
    let iter = values
        .iter()
        .filter(|x| *x % 2 == 0)
        .into_con_iter_x()
        .into_seq_iter();
    let result: Vec<_> = iter.map(|x| *x).collect();

    let expected: Vec<_> = values.into_iter().filter(|x| x % 2 == 0).collect();

    assert_eq!(result, expected);
}

#[test_matrix([1, 8, 64, 1025, 5483], [1, 10, 100, ])]
fn into_seq_iter_used(len: usize, take: usize) {
    let values: Vec<_> = (100..(100 + len)).collect();

    let iter = values.iter().filter(|x| *x % 2 == 0).into_con_iter_x();
    for _ in 0..take {
        _ = iter.next();
    }
    let result: Vec<_> = iter.into_seq_iter().map(|x| *x).collect();

    let mut iter = values.into_iter().filter(|x| x % 2 == 0);
    for _ in 0..take {
        _ = iter.next();
    }
    let expected: Vec<_> = iter.collect();

    assert_eq!(result, expected);
}

#[test_matrix([1, 8, 64, 1025, 5483], [1, 10, 100])]
fn buffered(len: usize, chunk_size: usize) {
    let values: Vec<_> = (100..(100 + len)).collect();
    let iter = values.iter().filter(|x| **x > 115).into_con_iter_x();
    let mut buffered = iter.buffered_iter_x(chunk_size);

    let mut current = 116;
    while let Some(chunk) = buffered.next_x() {
        for value in chunk {
            assert_eq!(value, &current);
            current += 1;
        }
    }

    assert_eq!(current, (100 + len).max(116));
}