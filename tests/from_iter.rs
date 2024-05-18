use orx_concurrent_iter::*;

#[test]
fn con_iter() {
    let values = ['a', 'b', 'c'];

    let con_iter = values.iter().into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().take(2).into_con_iter();
    assert_eq!(con_iter.next(), Some(&'a'));
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values.iter().skip(1).into_con_iter();
    assert_eq!(con_iter.next(), Some(&'b'));
    assert_eq!(con_iter.next(), Some(&'c'));
    assert_eq!(con_iter.next(), None);

    let con_iter = values
        .iter()
        .filter(|x| **x != 'a')
        .map(|x| x.to_string())
        .into_con_iter();
    assert_eq!(con_iter.next(), Some(String::from('b')));
    assert_eq!(con_iter.next(), Some(String::from('c')));
    assert_eq!(con_iter.next(), None);
}

#[test]
fn len() {
    let values = ['a', 'b', 'c', 'd'];
    let iter = values.iter();

    assert_eq!(iter.try_get_exact_len(), None);

    let con_iter = iter.into_con_iter();
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next();
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next_chunk(2);
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next_chunk(2);
    assert_eq!(con_iter.try_get_len(), None);

    _ = con_iter.next();
    assert_eq!(con_iter.try_get_len(), Some(0));

    _ = con_iter.next();
    assert_eq!(con_iter.try_get_len(), Some(0));
}

#[test]
fn into_seq_iter_unused() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter();
    let seq_iter = con_iter.into_seq_iter();

    assert_eq!(seq_iter.len(), 1024);
    for (i, x) in seq_iter.enumerate() {
        assert_eq!(x, i.to_string());
    }
}

#[test]
fn into_seq_iter_used_singly() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter();

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
    let con_iter = iter.into_con_iter();

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
        assert_eq!(x, (44 + 33 + i).to_string());
    }
}

#[test]
fn into_seq_iter_doc() {
    let iter = (0..1024).map(|x| x.to_string());
    let con_iter = iter.into_con_iter();

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
        assert_eq!(x, (num_used + i).to_string());
    }
}
