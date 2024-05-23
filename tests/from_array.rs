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
fn into_con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.into_con_iter();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.into_exact_con_iter();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.into_iter().take(2).into_con_iter();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn exact_len() {
    let values = ['a', 'b', 'c'];
    assert_eq!(3, values.exact_len())
}

#[test]
fn len() {
    let values = ['a', 'b', 'c', 'd'];

    let iter = values.into_con_iter();
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
    let mut array = [0; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.into_con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, i);
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let mut array = [0; 1024];
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

    assert_eq!(seq_iter.len(), 1024 - 114);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, 114 + i);
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let mut array = [0; 1024];
    for (i, x) in array.iter_mut().enumerate() {
        *x = i;
    }
    let con_iter = array.into_con_iter();

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
    let con_iter = array.into_con_iter();

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

#[test]
fn into_seq_iter_not_used() {
    let mut values = [0; 1024];
    for (i, x) in values.iter_mut().enumerate() {
        *x = i;
    }
    let iter = values.clone().into_con_iter().into_seq_iter();
    let result: Vec<_> = iter.collect();

    assert_eq!(result, values);
}

#[test_matrix([1, 10, 100])]
fn into_seq_iter_used(take: usize) {
    let mut values = [0; 1024];
    for (i, x) in values.iter_mut().enumerate() {
        *x = i;
    }

    let iter = values.clone().into_con_iter();
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
