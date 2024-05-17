use orx_concurrent_iter::*;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];
    let slice = values.as_slice();

    let con_iter = slice.con_iter().cloned();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = slice.into_con_iter().cloned();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = slice.into_exact_con_iter().cloned();
    assert_eq!(con_iter.next(), Some('a'));
    assert_eq!(con_iter.next(), Some('b'));
    assert_eq!(con_iter.next(), Some('c'));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = vec!['a', 'b', 'c', 'd'];
    let slice = values.as_slice();

    let iter = slice.con_iter().cloned();
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
    let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();
    let con_iter = slice.into_con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, &i.to_string());
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();
    let con_iter = slice.into_con_iter().cloned();

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
        assert_eq!(x, &(114 + i).to_string());
    }
}

#[test]
fn into_seq_iter_used_in_batches() {
    let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();
    let con_iter = slice.into_con_iter().cloned();

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
        assert_eq!(x, &(44 + 33 + i).to_string());
    }
}

#[test]
fn into_seq_iter_doc() {
    let vec: Vec<_> = (0..1024).map(|x| x.to_string()).collect();
    let slice = vec.as_slice();
    let con_iter = slice.into_con_iter().cloned();

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
        assert_eq!(x, &(num_used + i).to_string());
    }
}