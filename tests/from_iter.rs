use orx_concurrent_iter::*;
use test_case::test_matrix;

#[cfg(not(miri))]
const LEN: usize = 1024;
#[cfg(miri)]
const LEN: usize = 93;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.iter().iter_into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().take(2).iter_into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().skip(1).iter_into_con_iter();
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values
        .iter()
        .filter(|x| **x != 'a')
        .map(|x| x.to_string())
        .iter_into_con_iter();
    assert_eq!(con_iter.next(), Some(String::from('b')));
    assert_eq!(con_iter.next(), Some(String::from('c')));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = ['a', 'b', 'c', 'd'];
    let iter = values.iter();

    let iter = iter.iter_into_con_iter();
    assert_eq!(iter.try_get_len(), Some(4));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(3));

    _ = iter.chunk_puller(2).pull();
    assert_eq!(iter.try_get_len(), Some(1));

    _ = iter.chunk_puller(2).pull();
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));

    _ = iter.next();
    assert_eq!(iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let iter = (0..LEN).map(|x| x.to_string());
    let con_iter = iter.iter_into_con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), LEN);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, i.to_string());
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let iter = (0..LEN).map(|x| x.to_string());
    let con_iter = iter.iter_into_con_iter();

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
        assert_eq!(x, (114 + i).to_string());
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let iter = (0..LEN).map(|x| x.to_string());
    let con_iter = iter.iter_into_con_iter();

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
        assert_eq!(x, (44 + 33 + i).to_string());
    }
}

#[test]
fn into_seq_iter_doc() {
    let iter = (0..LEN).map(|x| x.to_string());
    let con_iter = iter.iter_into_con_iter();

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
        assert_eq!(x, (num_used + i).to_string());
    }
}

#[test_matrix([1, 8, 64, 1025, 5483])]
fn into_seq_iter_not_used(len: usize) {
    let values: Vec<_> = (100..(100 + len)).collect();
    let iter = values
        .iter()
        .filter(|x| *x % 2 == 0)
        .iter_into_con_iter()
        .into_seq_iter();
    let result: Vec<_> = iter.map(|x| *x).collect();

    let expected: Vec<_> = values.into_iter().filter(|x| x % 2 == 0).collect();

    assert_eq!(result, expected);
}

#[test_matrix([1, 8, 64, 1025, 5483], [1, 10, 100, ])]
fn into_seq_iter_used(len: usize, take: usize) {
    let values: Vec<_> = (100..(100 + len)).collect();

    let iter = values.iter().filter(|x| *x % 2 == 0).iter_into_con_iter();
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
    let iter = values.iter().filter(|x| **x > 115).iter_into_con_iter();
    let mut buffered = iter.chunk_puller(chunk_size);

    let mut current = 116;
    while let Some(chunk) = buffered.pull() {
        for value in chunk {
            assert_eq!(value, &current);
            current += 1;
        }
    }

    assert_eq!(current, (100 + len).max(116));
}
